# Design: OpenSpec Tasks Checkbox Desync Reconciliation & Warning

## System Architecture & Data Design

### 1. Reconciling Git Work against OpenSpec Tasks

The reconciliation mechanism operates within `src/commands/workflow.rs`, extending repository state probing (`probe_repo_state`) and context re-hydration without introducing external dependencies or modifying `state.json` schema boundaries.

```
┌────────────────────────────────────────────────────────┐
│                   Git Repository                      │
│  ┌───────────────────────┐   ┌───────────────────────┐  │
│  │ Dirty Working Tree    │   │ Branch Commits vs Base│  │
│  │ (status --porcelain)  │   │ (diff --name-only)    │  │
│  └───────────┬───────────┘   └───────────┬───────────┘  │
└──────────────┼───────────────────────────┼──────────────┘
               ▼                           ▼
        ┌─────────────────────────────────────────┐
        │       probe_feature_touched_files       │
        │  (Union set minus openspec/, lockfiles) │
        └────────────────────┬────────────────────┘
                             │
                             ▼
┌────────────────────────────────────────────────────────┐
│ openspec/changes/<feature>/tasks.md                    │
│  ┌──────────────────────────────────────────────────┐  │
│  │ - [ ] 1. Author canonical SKILL.md in `skills/`  │  │
│  │ - [ ] 2. Create `src/source/builtin_skills.rs`   │  │
│  └──────────────────────────┬───────────────────────┘  │
└─────────────────────────────┼──────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────┐
│             reconcile_tasks_with_git                  │
│  1. Extract path tokens from unchecked tasks           │
│  2. Exact / prefix / suffix match against touched      │
│  3. Fallback: completed==0 && touched has src/ / tests/│
│  4. Produce TaskDesyncReport                           │
└─────────────────────────────┬──────────────────────────┘
                              │
     ┌────────────────────────┼────────────────────────┐
     ▼                        ▼                        ▼
resume_lines             status_lines             doctor.rs
(! Warning banner)       (! Warning banner)       (doctor-warn: probe)
```

---

### 2. Core Data Structures

```rust
/// Match details for an unchecked task that correlates with modified files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDesyncMatch {
    pub task_index: usize,
    pub task_text: String,
    pub matched_files: Vec<String>,
}

/// Comprehensive report on tasks.md progress vs real git changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDesyncReport {
    pub feature: String,
    pub tasks_path: PathBuf,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub desynced_tasks: Vec<TaskDesyncMatch>,
    pub is_aggregate_desync: bool,
}

impl TaskDesyncReport {
    pub fn has_desync(&self) -> bool {
        !self.desynced_tasks.is_empty() || self.is_aggregate_desync
    }

    pub fn warning_line(&self) -> String {
        if !self.desynced_tasks.is_empty() {
            let count = self.desynced_tasks.len();
            let mut sample_files: Vec<String> = Vec::new();
            for m in &self.desynced_tasks {
                for f in &m.matched_files {
                    if !sample_files.contains(f) {
                        sample_files.push(f.clone());
                    }
                }
            }
            let preview = if sample_files.len() <= 2 {
                sample_files.join(", ")
            } else {
                format!("{}, +{} more", sample_files[..2].join(", "), sample_files.len() - 2)
            };
            format!(
                "! Warning: Tasks desync detected — {count} unchecked task(s) reference modified files ({preview}), but tasks.md shows {}/{} completed. Update tasks.md (- [x]) to reflect progress.",
                self.completed_tasks, self.total_tasks
            )
        } else if self.is_aggregate_desync {
            format!(
                "! Warning: Tasks desync detected — working tree / branch contains modified code, but tasks.md shows 0/{} completed. Update tasks.md (- [x]) to reflect progress.",
                self.total_tasks
            )
        } else {
            String::new()
        }
    }
}
```

---

### 3. Path Extraction Algorithm

Given an unchecked task string line (e.g. `- [ ] 2. Create `src/source/builtin_skills.rs` with fallback`), the tokenizer:
1. Regex matches backticked segments: `` `([^`]+)` ``.
2. If no backticks or in addition, splits on whitespace and cleans tokens (strips `(`, `)`, `:`, `,`, `.` from boundaries).
3. Identifies candidate paths matching:
   - Contains `/` (e.g. `src/commands/install.rs`, `tests/cli.rs`, `docs/plans/`).
   - Suffixes matching recognized project extensions: `.rs`, `.ts`, `.js`, `.json`, `.toml`, `.md`, `.sh`, `.yml`, `.yaml`.
4. Returns deduplicated `Vec<String>`.

---

### 4. Git Diff Sourcing Logic

`probe_feature_touched_files(repo_root: &Path) -> Vec<String>`:
1. Runs `probe_git_dirty_files(repo_root)` (`git status --porcelain=v1`) to capture uncommitted dirty files.
2. Runs:
   ```bash
   git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD main 2>/dev/null
   ```
   If a base commit is found, runs `git diff --name-only <base>...HEAD`.
3. Combines both lists into a deduplicated, sorted list.
4. Excludes `.git/`, `Cargo.lock`, `*.lock`, and anything starting with `openspec/` (since changes to the spec itself do not constitute task implementation).

---

### 5. Multi-Level Matching Rules

For each unchecked task:
- If any extracted path `p`:
  - `touched.contains(&p)` (Exact match)
  - `touched.iter().any(|f| f.starts_with(&p))` (Prefix/directory match, e.g. `src/commands/` or `skills/`)
  - `touched.iter().any(|f| f.ends_with(&format!("/{}", p)))` (Filename/basename match)
  Then `p` matches and is appended to `matched_files`.
- If matched, record `TaskDesyncMatch`.

Aggregate Fallback:
- If `desynced_tasks.is_empty()` AND `total_tasks > 0` AND `completed_tasks == 0`:
- If any touched file starts with `src/`, `tests/`, or `skills/`:
- Set `is_aggregate_desync = true`.

---

### 6. Integration Points

1. **`RepoState`**:
   Extend `RepoState` with:
   ```rust
   #[serde(skip_serializing_if = "Option::is_none")]
   pub task_desync: Option<TaskDesyncReport>,
   ```
2. **`probe_repo_state(ctx, wf)`**:
   Invokes `reconcile_tasks_with_git` when `openspec_context` is present.
3. **`resume_lines(ctx)`**:
   If `repo_state.task_desync` has desync, push `repo_state.task_desync.warning_line()` immediately under `tasks progress`.
4. **`status_lines(ctx)`**:
   Surface warning line in the FSM status display.
5. **`checkpoint_lines(ctx, ...)`**:
   Append warning line to checkpoint confirmation output if desync is present.
6. **`doctor.rs`**:
   Inspect adopted projects in `state.json` or current workspace:
   If `reconcile_tasks_with_git` detects desync, print:
   ```text
   doctor-warn: openspec tasks desync in '<feature>': <summary>
   ```
   Never fail `doctor` (does not append to `findings`, exits 0).
7. **`maybe_auto_checkpoint`**:
   Guard FSM: if `task_desync.has_desync()`, inhibit auto-checkpointing to `WorkflowStage::Verification`, `KnowledgeCapture`, or `GitShipping`.
