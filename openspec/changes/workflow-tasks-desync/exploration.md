# Exploration: OpenSpec Tasks Checkbox Desync Reconciliation

## 1. Technical Investigation & Current Code Paths

### 1.1 Existing Progress Deduction Logic
In `src/commands/workflow.rs`, the FSM stage is inferred by [`infer_stage_from_repo`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/workflow.rs#L725-L807) and context is gathered by [`probe_openspec_context_in`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/workflow.rs#L476-L550).

Both functions examine `openspec/changes/<feature>/tasks.md` using identical line-by-line counting:
```rust
for line in content.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
        completed_tasks += 1;
        total_tasks += 1;
    } else if trimmed.starts_with("- [ ]") {
        total_tasks += 1;
    }
}
```

Stage determination in `infer_stage_from_repo`:
- If `total_tasks > 0 && completed_tasks == 0` ➔ Infers `WorkflowStage::ExecutionPlan` (Stage 3).
- If `completed_tasks > 0 && completed_tasks < total_tasks` ➔ Infers `WorkflowStage::WorkTdd` (Stage 4).
- If `total_tasks > 0 && completed_tasks == total_tasks` ➔ Evaluates Stages 5 (Verify), 6 (Compound), or 7 (Ship).

### 1.2 The Silent Failure Mode
When an agent or developer executes `ce-work`:
1. The code changes are implemented in the working tree or committed on the feature branch.
2. If `tasks.md` is not updated with `- [x]`:
   - `infer_stage_from_repo` returns `Stage 3: ExecutionPlan`.
   - `resume_lines` outputs `tasks progress: 0/N completed ([x])`.
   - Pre-invocation hooks re-inject the Stage 3 prompt into the agent's context window.
   - The agent is instructed to plan the implementation, effectively restarting the task despite code already existing.
3. No warning is logged because `probe_repo_state` only checks `manifest_drift_count` (managed plugin files) and `adoption_status`, completely omitting correlation between project git modifications and `tasks.md`.

---

## 2. Reconciling Work Against `tasks.md`

### 2.1 Git Diff Sourcing
To know what work has occurred, we must collect all relevant modified files in the repository:
1. **Uncommitted Working Tree Modifications**:
   - Captured via `probe_git_dirty_files(repo_root)` (`git status --porcelain=v1`).
   - Yields unstaged and staged files.
2. **Committed Branch Modifications**:
   - On a feature branch (e.g. `feat/*`, `fix/*`), changes may already have been committed in atomic commits before session interruption.
   - Captured via:
     ```bash
     git diff --name-only $(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD main 2>/dev/null || echo HEAD~1)...HEAD
     ```
   - Filtering: Ignore changes within `openspec/` itself, lockfiles, and `.git/`.
3. **Union Set**: Combine uncommitted modified files and branch-committed files into `HashSet<String>` representing all `touched_files` in the feature scope.

### 2.2 Task Text Parsing & Path Extraction
Tasks in OpenSpec `tasks.md` are typically structured as:
- `- [ ] 1. Author canonical skills/sequential-thinking/SKILL.md in repository root`
- `- [ ] Implement CodexAdapter in src/harness/codex.rs`
- `- [ ] Add integration tests in tests/cli.rs`
- `- [ ] Update documentation in docs/user-guide/`

We extract target references using a deterministic regex/scanner:
1. **Backticked strings**: Extracts code blocks matching `` `([^`]+)` ``.
2. **Path-like tokens**: Scans space-delimited words for file paths:
   - Containing `/` (e.g. `src/commands/workflow.rs`, `tests/cli.rs`, `docs/plans/`).
   - Ending in standard source extensions (`.rs`, `.ts`, `.js`, `.json`, `.toml`, `.md`, `.sh`, `.yml`, `.yaml`).
3. **Normalizing**: Strips leading `./` or trailing punctuation (`,`, `:`, `.`).

### 2.3 Correlation & Match Classification
For each unchecked task `- [ ] <description>`:
1. Extract candidate paths `P = {p_1, p_2, ...}`.
2. Test against `touched_files`:
   - **Exact Match**: `touched_files.contains(p)`.
   - **Prefix Match**: If `p` is a directory (e.g. `src/commands/` or `src/harness/`), any touched file starting with `p`.
   - **Suffix Match**: If `p` is a relative path fragment like `workflow.rs` or `cli.rs`, any touched file ending with `p`.
3. If any candidate matches, the task is flagged as **DesyncedTask** (work appears done in code, but task is unchecked).

### 2.4 Fallback Heuristic (Tasks Lacking Explicit Paths)
When tasks are written abstractly (e.g., `- [ ] Setup database migration`, `- [ ] Write regression tests`):
- If candidate paths `P` are empty for all unchecked tasks:
- We check if `completed_tasks == 0` while `touched_files` contains source files under `src/` or `tests/`.
- If non-spec source code was modified, we classify this as an **AggregateDesync** (work occurred on this change, but 0 tasks are checked off).

---

## 3. Evaluated Notification Channels

| Channel | Visibility | Non-Blocking | Implementation Location |
|---|---|---|---|
| **Context Re-hydration (`resume_lines`)** | High (injected into agent context) | Yes (text banner) | `src/commands/workflow.rs:311` |
| **Workflow Status (`status_lines`)** | High (visible in CLI `workflow status` and TUI) | Yes (text lines) | `src/commands/workflow.rs:182` |
| **Health Check (`ce-ai doctor`)** | High (runs in diagnostics and CI) | Yes (`doctor-warn:`, exit 0) | `src/commands/doctor.rs` |
| **Workflow Checkpoint (`checkpoint_lines`)** | Medium (surfaced when saving checkpoints) | Yes (warning in output) | `src/commands/workflow.rs:224` |
| **FSM Transition Guard (`maybe_auto_checkpoint`)** | High (prevents false FSM stage jumps) | Yes (prevents auto-advance) | `src/commands/workflow.rs:900` |

### Architectural Decision
- Emitting visible warnings across `resume_lines`, `status_lines`, `checkpoint_lines`, and `doctor` satisfies all user and issue requirements without breaking automation.
- In `maybe_auto_checkpoint`, prevent auto-advancing to `Verification` or `Ship` when `completed_tasks < total_tasks` and desync is present.
- Never hard-block manual checkpoint commands (`ce-ai workflow checkpoint`).

---

## 4. Proposed Data Structures

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDesyncMatch {
    pub task_index: usize,
    pub task_text: String,
    pub matched_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDesyncReport {
    pub feature: String,
    pub tasks_path: PathBuf,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub desynced_tasks: Vec<TaskDesyncMatch>,
    pub is_aggregate_desync: bool,
}
```
