# Design: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## 1. System Architecture

```
+-----------------------------------------------------------------------------------+
| Harness Touchpoints                                                               |
| - Turn-0: PreInvocation (Agy), before_agent_start (Pi), session.created (OpenCode)|
| - Turn-End: Stop (Agy), agent_end (Pi), session.idle (OpenCode)                   |
| - Pre-Compact: session_before_compact (Pi), experimental.compacting (OpenCode)    |
| - Explicit Commands: ce-ai workflow resume / status / doctor                      |
+----------------------------------------+------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Stage Inference Engine (src/commands/workflow.rs)                                 |
|                                                                                   |
| 1. Transitory Git Guard: abort if .git/rebase-merge, MERGE_HEAD, etc.             |
| 2. Branch Resolution: git branch --show-current                                  |
| 3. Feature Sanitization: sanitize_feature_name(branch)                            |
| 4. OpenSpec Probe: inspect openspec/changes/<feature>/{proposal,spec,tasks}.md    |
| 5. Git Status Probe: dirty files, solutions/, open PR                             |
| 6. Monotonic Stage Calculation: determine target WorkflowStage (1..7)             |
+----------------------------------------+------------------------------------------+
                                         |
                                         v
+-----------------------------------------------------------------------------------+
| Optimistic Concurrency & State Persistence (src/state/state.rs)                  |
|                                                                                   |
| 1. Read on-disk state.json immediately before mutation (CAS)                     |
| 2. Key: <canonical_root>::<branch> (fallback: <canonical_root>)                  |
| 3. Verify target_stage > current_stage && current_stage.can_transition_to(...)    |
| 4. Never overwrite Manual source with Inferred source at same/lower stage        |
| 5. Persist atomically via write_atomic (temp file + rename)                       |
+-----------------------------------------------------------------------------------+
```

---

## 2. Data Structures & Schema Updates

### 2.1 `WorkflowSource` and `WorkflowState`
In `src/state/state.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowSource {
    Manual,
    Inferred,
}

impl Default for WorkflowSource {
    fn default() -> Self {
        WorkflowSource::Manual
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    pub stage: WorkflowStage,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_name: Option<String>,
    pub updated_at: String,
    #[serde(default)]
    pub source: WorkflowSource,
}
```

### 2.2 Branch-Aware Workspace Keying
In `src/state/state.rs`:

```rust
impl State {
    /// Constructs a composite key combining canonical root and git branch.
    /// Falls back to canonical root if branch is None or empty.
    pub fn workspace_branch_key(root: &Path, branch: Option<&str>) -> String {
        let canonical_root = Self::normalize_workspace_key(root);
        match branch.filter(|b| !b.trim().is_empty()) {
            Some(b) => format!("{canonical_root}::{b}"),
            None => canonical_root,
        }
    }

    /// Resolves active workflow with hierarchical fallback:
    /// 1. <canonical_root>::<branch>
    /// 2. <canonical_root> (legacy workspace key)
    /// 3. Top-level scalar workflow (legacy global)
    pub fn current_workflow_for_branch(&self, root: &Path, branch: Option<&str>) -> Option<WorkflowState> {
        let canonical_root = Self::normalize_workspace_key(root);
        if let Some(b) = branch.filter(|b| !b.trim().is_empty()) {
            let key = format!("{canonical_root}::{b}");
            if let Some(wf) = self.workflows.get(&key) {
                return Some(wf.clone());
            }
        }
        if let Some(wf) = self.workflows.get(&canonical_root) {
            return Some(wf.clone());
        }
        self.workflow.clone()
    }
}
```

---

## 3. Optimistic Concurrency Control (CAS)

In `src/state/state.rs`:

```rust
impl State {
    /// Mutates workflow state using read-before-write compare-and-swap.
    /// Reloads state.json from disk immediately before mutation.
    pub fn atomic_update_workflow<F>(
        state_path: &Path,
        root: &Path,
        branch: Option<&str>,
        mutator: F,
    ) -> Result<WorkflowState, CeError>
    where
        F: FnOnce(&Option<WorkflowState>) -> Result<Option<WorkflowState>, CeError>,
    {
        let mut state = if state_path.exists() {
            State::load(state_path)?
        } else {
            State::default()
        };

        let current = state.current_workflow_for_branch(root, branch);
        let updated_opt = mutator(&current)?;

        if let Some(new_wf) = updated_opt {
            let key = Self::workspace_branch_key(root, branch);
            state.workflows.insert(key, new_wf.clone());
            state.workflow = Some(new_wf.clone());
            state.save(state_path)?;
            Ok(new_wf)
        } else {
            Ok(current.unwrap_or_default())
        }
    }
}
```

---

## 4. Stage Inference Engine

In `src/commands/workflow.rs`:

### 4.1 Transitory Git State Detection
```rust
pub fn is_transitory_git_state(repo_root: &Path) -> bool {
    let git_dir = repo_root.join(".git");
    git_dir.join("rebase-merge").exists()
        || git_dir.join("rebase-apply").exists()
        || git_dir.join("CHERRY_PICK_HEAD").exists()
        || git_dir.join("MERGE_HEAD").exists()
}
```

### 4.2 Sanitization of Feature / Branch Names
```rust
pub fn sanitize_feature_name(branch: &str) -> String {
    let stripped = branch
        .trim_start_matches("refs/heads/")
        .trim_start_matches("feature/")
        .trim_start_matches("feat/")
        .trim_start_matches("fix/");
    stripped
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect()
}
```

### 4.3 Stage Deduction Matrix
```rust
pub fn infer_stage_from_repo(repo_root: &Path, branch: Option<&str>) -> Option<(WorkflowStage, String, Option<String>)> {
    if is_transitory_git_state(repo_root) {
        return None;
    }

    let raw_branch = branch.unwrap_or("").trim();
    let sanitized_feature = if !raw_branch.is_empty() {
        Some(sanitize_feature_name(raw_branch))
    } else {
        None
    };

    let openspec_info = probe_openspec_context_in(repo_root, &None);

    // Stage 7: Ship (PR open or branch pushed)
    // Stage 6: Compound (solutions/ modified on branch)
    // Stage 5: Verify (all tasks completed)
    // Stage 4: Work/TDD (partial tasks completed OR dirty changes on fix/feat branch)
    // Stage 3: Execution Plan (tasks.md exists, 0 completed)
    // Stage 2: OpenSpec Definition (proposal.md + spec.md exist)
    // Stage 1: Ideation (brainstorms or ideation present)
    // ...
}
```

---

## 5. Harness Hook Lifecycle & Implementation

### 5.1 Google Antigravity (`src/harness/agy.rs`)
- In `.agents/hooks.json` under `"compound-engineering"`:
  - `"PreInvocation"`: `ce-ai workflow resume --pre-invocation` (Turn-0 context delivery).
  - `"Stop"`: `ce-ai workflow resume` (Turn-end auto-checkpoint).
- `remove_pre_invocation_hook` updated to symmetrically strip BOTH `"PreInvocation"` and `"Stop"`.

### 5.2 Pi (`src/harness/pi.rs`, `PI_EXTENSION_CONTENT`)
- In `.pi/extensions/compound-engineering.ts`:
  - `session_start` & `before_agent_start`: Turn-0 drift delivery.
  - `agent_end`: Turn-end auto-checkpoint.
  - `session_before_compact`: Pre-compaction checkpoint.
- Embed `// ce-ai:hook v=2` header.

### 5.3 OpenCode (`src/opencode/plugins.rs`, `BUILTIN_LOADER`)
- In `.opencode/plugins/compound-engineering.js`:
  - `event.type === "session.created"`: Turn-0 context injection.
  - `event.type === "session.idle"`: Turn-end auto-checkpoint.
  - `experimental.session.compacting`: Pre-compaction checkpoint.

### 5.4 Claude, Codex, Cursor, Copilot Symmetries
- In `claude.rs`, `cursor.rs`, `copilot.rs`, `codex.rs`:
  - Symmetrically remove any newly introduced hook keys in `remove_*_hook`.

---

## 6. Versioning, Upgrade & Configuration

1. **`init-prj --force`**: Checks hook versions. If stale or `--force` is specified, rewrites/refreshes the hook definitions.
2. **`sync` and `upgrade`**: Wire `verify_and_refresh_harness_hooks` into `ce-ai sync` and `ce-ai upgrade` to ensure existing projects adopt new hooks without manual re-adoption.
3. **Opt-Out Configuration**:
   Support `auto_checkpoint: bool` (default `true`). If set to `false` in config or `--no-auto-checkpoint` CLI flag, automatic inference skips persisting to `state.json`.
