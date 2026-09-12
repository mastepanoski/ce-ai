# Technical Design: Documentation Technical Debt Diagnostic Engine & Probes

## 1. Data Models & Structs

### A. Substrate Tri-State (`ProbeStatus<T>`)
Located in `src/commands/workflow.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data", rename_all = "lowercase")]
pub enum ProbeStatus<T> {
    Clean,
    Debt(T),
    Unknown,
}

impl<T> ProbeStatus<T> {
    pub fn is_debt(&self) -> bool {
        matches!(self, Self::Debt(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}
```

### B. Finding Types
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenSpecDesyncFinding {
    pub feature: String,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub reason: DesyncReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesyncReason {
    ParentTasksCompleteSubtasksOpen,
    CodeMergedToMain,
    ReleaseVersionSurpassed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StalePendingFinding {
    pub feature: String,
    pub days_inactive: u32,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub source: InactivitySource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InactivitySource {
    GitCommitDate,
    FilesystemMtime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionDriftFinding {
    pub solution_path: String,
    pub dead_paths: Vec<String>,
    pub missing_frontmatter_fields: Vec<String>,
}
```

### C. Umbrella Report (`DocDebtReport`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocDebtReport {
    pub git_available: bool,
    pub openspec_desync: ProbeStatus<Vec<OpenSpecDesyncFinding>>,
    pub stale_pending: ProbeStatus<Vec<StalePendingFinding>>,
    pub solution_drift: ProbeStatus<Vec<SolutionDriftFinding>>,
}
```

### D. Extended `RepoState`
In `src/commands/workflow.rs`:
```rust
pub struct RepoState {
    ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_debt: Option<DocDebtReport>,
}
```

### E. Workspace Override Configuration (`DocHygieneConfig`)
In `src/state/state.rs`:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocHygieneConfig {
    #[serde(default = "default_stale_spec_days")]
    pub stale_spec_days: u32,
    #[serde(default = "default_true")]
    pub check_solution_paths: bool,
    #[serde(default = "default_true")]
    pub require_solution_frontmatter: bool,
}

fn default_stale_spec_days() -> u32 { 21 }
fn default_true() -> bool { true }

impl Default for DocHygieneConfig {
    fn default() -> Self {
        Self {
            stale_spec_days: 21,
            check_solution_paths: true,
            require_solution_frontmatter: true,
        }
    }
}
```

In `State`:
```rust
pub struct State {
    ...
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_hygiene: Option<DocHygieneConfig>,
}
```

## 2. Probe Implementation Signatures

In `src/commands/workflow.rs`:
```rust
pub fn probe_doc_debt(
    repo_root: &Path,
    git_branch: Option<&str>,
    config: &DocHygieneConfig,
) -> DocDebtReport;

pub fn probe_openspec_desync(
    repo_root: &Path,
    git_available: bool,
) -> ProbeStatus<Vec<OpenSpecDesyncFinding>>;

pub fn probe_stale_pending_openspecs(
    repo_root: &Path,
    stale_days: u32,
    git_available: bool,
) -> ProbeStatus<Vec<StalePendingFinding>>;

pub fn probe_solution_drift(
    repo_root: &Path,
    config: &DocHygieneConfig,
) -> ProbeStatus<Vec<SolutionDriftFinding>>;
```

## 3. CLI & Turn-0 Output Contracts

### A. Turn-0 Banner Formatting
In `render_repo_state_banner`:
```
doc debt: clean
```
When findings exist:
```
doc debt: 2 unarchived (code merged), 1 stale spec (34d), 2 dead solution links
```
When Git is absent:
```
doc debt: 1 stale spec [git: n/a], 2 dead solution links
```

### B. `ce-ai doctor` Output Contract
In `src/commands/doctor.rs`:
```
doctor-warn: openspec change 'doctor-kimi-marketplace-divergence' is complete with open subtasks (progress: 5/20) — run 'ce-ai archive doctor-kimi-marketplace-divergence --auto-mark' or 'ce-ai archive doctor-kimi-marketplace-divergence --status "..."'
doctor-warn: openspec change 'linux-musl-static-release' has been pending for 34 days with no git commits (progress: 9/10) — resume, shelve, or archive with '--status "superseded"'
doctor-warn: solution 'architecture/old.md' references non-existent path 'src/old_module.rs'
doctor-warn: solution 'bugfixes/fix.md' missing required YAML frontmatter: applies_when
```
