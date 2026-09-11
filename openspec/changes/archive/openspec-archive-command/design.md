# Design: OpenSpec Change Archival CLI Command & Ledger Synchronization

## 1. System Architecture & Component Interactions

The archive command bridges the diagnostic probe (`probe_unarchived_completed_changes`) with filesystem execution, git index staging, and ledger maintenance:

```
[CLI Dispatch: main.rs / registry.rs]
   │
   ├─► Commands::Archive(args) ──────────┐
   │                                     ▼
   └─► Commands::Workflow(Action::Archive(args))
                                         │
                                         ▼
                      [archive::run(ctx, args)]
                                         │
               ┌─────────────────────────┴─────────────────────────┐
               ▼                                                   ▼
       [Single Feature]                                       [--all Flag]
  Resolve feature name or active                         Probe all completed changes
  feature from state.json                                via probe_unarchived_completed_changes
               │                                                   │
               └─────────────────────────┬─────────────────────────┘
                                         ▼
                     [validate_and_archive_feature]
                                         │
                      1. Check source exists: openspec/changes/<f>
                      2. Check collision: openspec/changes/archive/<f>
                      3. Check criteria:
                         - Criterion 1: completed == total > 0
                         - Criterion 2: --status "<evidence>"
                      4. Check git dirty files in target directory
                      5. Prepend STATUS to tasks.md (if Criterion 2)
                      6. Move directory (git mv or fs rename + stage)
                                         │
                                         ▼
                       [sync_archive_readme_ledger]
                      Append audit entry to archive/README.md
                                         │
                                         ▼
                        [reconcile_state_active_feature]
                      Clear active feature in state.json if matches
```

## 2. CLI Interface & Structs

### Command Definitions (`src/commands/workflow.rs` & `src/commands/registry.rs`)

```rust
#[derive(clap::Args, Debug, Clone, Default)]
pub struct ArchiveArgs {
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

### Action Enum Extension (`src/commands/workflow.rs`)

```rust
pub enum Action {
    // Existing actions: Status, Checkpoint, Resume, ReviewReceipt...
    /// Archive completed OpenSpec change packages to openspec/changes/archive/.
    Archive(ArchiveArgs),
}
```

### Top-Level Registry Alias (`src/commands/registry.rs`)

```rust
pub enum Commands {
    // Existing commands...
    /// Archive completed OpenSpec change packages to openspec/changes/archive/ (alias for workflow archive).
    Archive(crate::commands::workflow::ArchiveArgs),
}
```

## 3. Data Schema & Core Engine Functions

### Outcome Model (`src/commands/workflow.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveCriterion {
    Mechanical { completed: usize, total: usize },
    StatusAttested { status: String, completed: usize, total: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveOutcome {
    pub feature: String,
    pub source_path: PathBuf,
    pub dest_path: PathBuf,
    pub criterion: ArchiveCriterion,
}
```

### Validation & Execution Pipeline

```rust
pub fn validate_and_archive_feature(
    repo_root: &Path,
    feature: &str,
    status: Option<&str>,
    dry_run: bool,
) -> Result<ArchiveOutcome, CeError>
```

1. **Existence Check:** `openspec/changes/<feature>` must exist and be a directory (`CeError::Usage` if absent).
2. **Collision Check:** `openspec/changes/archive/<feature>` must NOT exist (`CeError::State` if collision occurs).
3. **Criteria Evaluation:**
   - Parse `tasks.md` via `count_task_checkboxes`.
   - If `completed == total && total > 0`: Satisfies `ArchiveCriterion::Mechanical`.
   - Else if let Some(ref_text) = status:
     - Verify `!ref_text.trim().is_empty()` and minimum length >= 5 characters.
     - Satisfies `ArchiveCriterion::StatusAttested`.
   - Else: Fail with `CeError::Verification(format!("openspec change '{}' has {} open task(s) ({}/{} completed); complete all tasks or supply --status '<evidence>'", feature, total - completed, completed, total))`.
4. **Git Dirtiness Guard:**
   - Inspect git porcelain status for files under `openspec/changes/<feature>`.
   - If modified files exist outside `tasks.md`, return `CeError::Verification("feature directory contains uncommitted modifications outside tasks.md")`.
5. **Execution (unless `dry_run`):**
   - If `Criterion::StatusAttested`, prepend `> STATUS: <status>\n\n` to `tasks.md` using `crate::state::write_atomic`.
   - Ensure parent directory `openspec/changes/archive` exists.
   - Try `git mv openspec/changes/<feature> openspec/changes/archive/<feature>`.
   - If git fails or not a git worktree, fallback to `std::fs::rename` followed by `git add -A` if git is present.

### Ledger Synchronization (`sync_archive_readme_ledger`)

```rust
pub fn sync_archive_readme_ledger(
    repo_root: &Path,
    outcomes: &[ArchiveOutcome],
    dry_run: bool,
) -> Result<(), CeError>
```

- Target file: `openspec/changes/archive/README.md`.
- Formats an audit record:
  - For single or batch moves, appends under `Historical notes:` with current date, count of archived folders, and references.
  - Keeps formatting consistent with existing v1.20.1 / v1.44.2 sweep notes.

### State Active Feature Reconciliation

```rust
pub fn reconcile_state_active_feature(
    ctx: &Context,
    archived_features: &[String],
    dry_run: bool,
) -> Result<(), CeError>
```

- Loads `state.json`.
- If `state.workflow` exists and its `feature` matches any archived feature, clears `state.workflow.feature = None` (or marks completed).
- Saves using `crate::state::write_atomic`.

## 4. Exit Code Contract Compliance

- **`0` Success:** Feature(s) archived cleanly or dry-run finished.
- **`1` Runtime:** Unexpected process or git subshell error.
- **`2` Usage:** Missing feature name when no active feature exists, or non-existent source directory.
- **`3` State:** Target archive directory already exists (collision prevention).
- **`4` IO:** Filesystem read/write/rename failure.
- **`6` Verification:** Uncompleted tasks without `--status`, or dirty working tree in feature directory.
