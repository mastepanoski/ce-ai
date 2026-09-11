# Design: Deterministic Detection of Unarchived Completed OpenSpec Changes

## Architecture & Data Flow

```
   openspec/changes/
         │
         ├── [scan direct child directories, skip "archive"]
         ▼
   count_task_checkboxes(&tasks_path)
         │  (graceful degradation on read error / missing file)
         ▼
   probe_unarchived_completed_changes(&repo_root) -> Vec<UnarchivedChange>
         │
         ▼
   probe_repo_state(ctx, wf) -> RepoState
         │
         ├──► resume_lines(ctx)
         │       └─ "openspec ledger: clean (0 pending archival)"
         │          OR "openspec ledger: ! N change(s) complete but not archived — run 'ce-ai doctor' for details"
         ├──► status_lines(ctx) & checkpoint_lines(ctx)
         │       └─ "! Warning: N OpenSpec change(s) complete but not archived — run 'ce-ai doctor' for details"
         ├──► resume --json (RepoState JSON payload & additionalContext)
         │
         ▼
   doctor::run(ctx)
         └─ "doctor-warn: openspec change '<feature>' is complete (N/N tasks) but not archived — see openspec/changes/archive/README.md"
```

## Data Structures

### `UnarchivedChange` (`src/commands/workflow.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnarchivedChange {
    pub feature: String,
    pub completed_tasks: usize,
    pub total_tasks: usize,
}
```

### `RepoState` Extension (`src/commands/workflow.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoState {
    pub git_branch: Option<String>,
    pub head_sha: Option<String>,
    pub is_git_clean: bool,
    pub modified_files: Vec<String>,
    pub manifest_drift_count: usize,
    pub adoption_status: Option<AdoptionBlockStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openspec_context: Option<OpenSpecContextInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_desync: Option<TaskDesyncReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unarchived_completed_changes: Vec<UnarchivedChange>,
}
```

## Algorithms & Logic

### 1. Checkbox Counting Extraction
```rust
pub fn count_task_checkboxes(tasks_path: &Path) -> (usize, usize) {
    let mut completed_tasks = 0;
    let mut total_tasks = 0;
    if let Ok(content) = std::fs::read_to_string(tasks_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                completed_tasks += 1;
                total_tasks += 1;
            } else if trimmed.starts_with("- [ ]") {
                total_tasks += 1;
            }
        }
    }
    (completed_tasks, total_tasks)
}
```

### 2. Repo-Wide Directory Scanning
```rust
pub fn probe_unarchived_completed_changes(repo_root: &Path) -> Vec<UnarchivedChange> {
    let openspec_dir = repo_root.join("openspec").join("changes");
    let mut completed_changes = Vec::new();
    let entries = match std::fs::read_dir(&openspec_dir) {
        Ok(read) => read,
        Err(_) => return completed_changes,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if dir_name == "archive" {
            continue;
        }

        let tasks_path = path.join("tasks.md");
        if !tasks_path.is_file() {
            continue;
        }

        let (completed, total) = count_task_checkboxes(&tasks_path);
        if total > 0 && completed == total {
            completed_changes.push(UnarchivedChange {
                feature: dir_name.to_string(),
                completed_tasks: completed,
                total_tasks: total,
            });
        }
    }

    completed_changes.sort_by(|a, b| a.feature.cmp(&b.feature));
    completed_changes
}
```

## Surface Rendering Contracts

1. **`resume_lines`**:
   Rendered in `"== [Environment State & Drift Status] =="` immediately following `adoption_status`:
   - If empty: `  openspec ledger: clean (0 pending archival)`
   - If non-empty: `  openspec ledger: ! {count} change(s) complete but not archived — run 'ce-ai doctor' for details`

2. **`status_lines` & `checkpoint_lines`**:
   If non-empty, appended right after `task_desync.warning_line()`:
   - `! Warning: {count} OpenSpec change(s) complete but not archived — run 'ce-ai doctor' for details`

3. **`ce-ai doctor`**:
   Iterated after the active tasks desync probe:
   - `doctor-warn: openspec change '{feature}' is complete ({completed}/{total} tasks) but not archived — see openspec/changes/archive/README.md`
   - Non-fatal: does not populate `findings` and exits 0 in the absence of other errors.
