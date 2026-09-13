# Technical Design: Generational Archive Compaction & Milestone Rollups

## 1. System Architecture

Archive compaction provides automated lifecycle management for completed OpenSpec changes stored in `openspec/changes/archive/`.

```
[ ce-ai archive compact ]
          │
          ├──► 1. Collect loose directories in openspec/changes/archive/ (skip milestones/)
          ├──► 2. Resolve dates (directory name prefix ➔ git commit log ➔ mtime)
          ├──► 3. Filter candidates (date < --before, threshold checks)
          ├──► 4. Group candidates into Milestones (quarterly YYYY-QX or explicit --milestone)
          │
     ┌────┴───────────────────────────┐
     │  Dry Run?                      │
     │  ├── YES: Output preview plan  │
     │  └── NO: Execute mutations     │
     └────┬───────────────────────────┘
          │
          ├──► 5. Write openspec/changes/archive/milestones/<milestone>.md
          ├──► 6. Package raw directories into milestones/archive-<milestone>.tar.gz
          ├──► 7. Verify tarball integrity and entry counts
          ├──► 8. Safely remove loose directories from openspec/changes/archive/ (unless --keep-loose)
          └──► 9. Update openspec/changes/archive/README.md with milestone ledger table
```

## 2. CLI Interface & Command Hierarchy

`ce-ai archive` expands to support the `compact` subcommand while preserving direct feature archival:

```bash
# Preview compaction without modifying disk
ce-ai archive compact --dry-run

# Compact archives older than a specific cutoff date
ce-ai archive compact --before 2026-08-01

# Compact matching archives into a specific named milestone
ce-ai archive compact --milestone 2026-Q2

# Compact only if loose package count exceeds threshold
ce-ai archive compact --threshold 30

# Preserve loose directories while generating rollup and tarball
ce-ai archive compact --keep-loose

# Generate rollup markdown without creating tarball
ce-ai archive compact --no-tarball

# Full workflow namespace alias
ce-ai workflow archive compact --dry-run
```

## 3. Data Structures & Schemas

### 3.1 Workspace Configuration (`DocHygieneConfig` in `src/state/state.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocHygieneConfig {
    #[serde(default = "default_stale_spec_days")]
    pub stale_spec_days: u32,
    #[serde(default = "default_true")]
    pub check_solution_paths: bool,
    #[serde(default = "default_true")]
    pub require_solution_frontmatter: bool,
    #[serde(default = "default_archive_compaction_threshold")]
    pub archive_compaction_threshold: u32,
}

fn default_archive_compaction_threshold() -> u32 {
    30
}
```

### 3.2 CLI Command Arguments (`src/commands/workflow.rs`)
```rust
#[derive(clap::Subcommand, Debug, Clone)]
pub enum ArchiveSubcommand {
    /// Compact aged archive packages into quarterly or milestone rollups.
    Compact(CompactArgs),
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct CompactArgs {
    /// Only compact archives created or dated before this date (YYYY-MM-DD).
    #[arg(long)]
    pub before: Option<String>,

    /// Explicit milestone name (defaults to auto-quarter e.g. '2026-Q3').
    #[arg(long)]
    pub milestone: Option<String>,

    /// Maximum uncompacted package count before compaction is triggered.
    #[arg(long)]
    pub threshold: Option<u32>,

    /// Preview compaction actions without modifying files or removing directories.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Do not generate a compressed tarball of the raw archives.
    #[arg(long, default_value_t = false)]
    pub no_tarball: bool,

    /// Keep loose directories after generating rollup and tarball (do not prune).
    #[arg(long, default_value_t = false)]
    pub keep_loose: bool,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct ArchiveArgs {
    #[command(subcommand)]
    pub subcommand: Option<ArchiveSubcommand>,

    /// Target change folder name to archive (defaults to active feature from state.json).
    pub feature: Option<String>,

    /// Archive all completed change folders detected in openspec/changes/.
    #[arg(long, default_value_t = false)]
    pub all: bool,

    /// Preview intended moves and ledger updates without modifying disk or git.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Criterion 2 STATUS attestation for features with incomplete tasks.
    #[arg(long)]
    pub status: Option<String>,
}
```

### 3.3 Diagnostic Report Finding (`DocDebtReport`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveCompactionFinding {
    pub uncompacted_count: usize,
    pub threshold: u32,
    pub oldest_package: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocDebtReport {
    pub git_available: bool,
    pub openspec_desync: ProbeStatus<Vec<OpenSpecDesyncFinding>>,
    pub stale_pending: ProbeStatus<Vec<StalePendingFinding>>,
    pub solution_drift: ProbeStatus<Vec<SolutionDriftFinding>>,
    pub archive_compaction: ProbeStatus<ArchiveCompactionFinding>,
}
```

### 3.4 In-Memory Compaction Models
```rust
pub struct CompactionCandidate {
    pub folder_name: String,
    pub feature_name: String,
    pub path: PathBuf,
    pub date: chrono::NaiveDate,
    pub tasks_progress: (usize, usize),
    pub proposal_summary: String,
    pub spec_summary: String,
}

pub struct MilestonePlan {
    pub milestone_name: String,
    pub candidates: Vec<CompactionCandidate>,
    pub rollup_path: PathBuf,
    pub tarball_path: Option<PathBuf>,
}
```

## 4. Tarball Compression & Extraction Verification

Compression uses `tar::Builder` combined with `flate2::write::GzEncoder`:
```rust
pub fn create_milestone_tarball(
    archive_dir: &Path,
    tarball_path: &Path,
    candidates: &[CompactionCandidate],
) -> Result<(), CeError> {
    let file = std::fs::File::create(tarball_path)?;
    let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    for cand in candidates {
        let rel_name = &cand.folder_name;
        tar.append_dir_all(rel_name, &cand.path)?;
    }
    tar.finish()?;
    Ok(())
}
```

Integrity verification checks:
1. Destination tarball file exists and size > 0.
2. Uncompressed entry inspection verifies all candidate directories are present as prefixes in the tarball.

## 5. Ledger Synchronization (`openspec/changes/archive/README.md`)

The archive ledger is updated by maintaining a dedicated markdown table:
```markdown
## Compacted Milestones

| Milestone | Date Period | Changes Compacted | Rollup Summary | Raw Archive |
| :--- | :--- | :--- | :--- | :--- |
| 2026-Q1 | <= 2026-03-31 | 35 | [milestones/2026-Q1.md](milestones/2026-Q1.md) | [milestones/archive-2026-Q1.tar.gz](milestones/archive-2026-Q1.tar.gz) |
```
Existing sections (triage, criteria, historical notes) are strictly preserved.

## 6. Doctor Health Check & Turn-0 Output

1. **Doctor Warning:**
   ```
   doctor-warn: archive has 109 uncompacted packages (>30 threshold); run 'ce-ai archive compact' to roll up aged changes into milestone summaries
   ```
2. **Turn-0 Summary Banner:**
   ```
   doc debt: 109 archive pkgs (>30), 2 unarchived (code merged)
   ```
