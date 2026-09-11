# Design: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## 1. System Architecture

```
+---------------------------------------------------------------------------------------------------+
| Harness Touchpoints (Complete 7-Harness Matrix)                                                   |
| - Turn-0: SessionStart (Claude/Codex/Cursor/Copilot), PreInvocation (Agy),                       |
|           before_agent_start (Pi), session.created (OpenCode)                                     |
| - Turn-End: Stop (Claude/Codex/Cursor/Agy), agent_end (Pi), session.idle (OpenCode),               |
|             postToolUse (Copilot)                                                                 |
| - Pre-Compact: PreCompact (Claude/Codex), session_before_compact (Pi),                             |
|                experimental.session.compacting (OpenCode)                                         |
| - Explicit Commands: ce-ai workflow resume / status / doctor                                      |
+-------------------------------------------------+-------------------------------------------------+
                                                  |
                                                  v
+---------------------------------------------------------------------------------------------------+
| Stage Inference Engine (src/commands/workflow.rs)                                                 |
|                                                                                                   |
| 1. Transitory Git Guard: abort if .git/rebase-merge, MERGE_HEAD, etc.                             |
| 2. Branch Resolution: git branch --show-current                                                  |
| 3. Feature Sanitization: sanitize_feature_name(branch)                                            |
| 4. OpenSpec Probe: inspect openspec/changes/<feature>/{proposal,spec,tasks}.md                    |
| 5. Git Status Probe: dirty files, solutions/, open PR                                             |
| 6. Monotonic Stage Calculation: determine target WorkflowStage (1..7)                             |
+-------------------------------------------------+-------------------------------------------------+
                                                  |
                                                  v
+---------------------------------------------------------------------------------------------------+
| Optimistic Concurrency & State Persistence (src/state/state.rs)                                  |
|                                                                                                   |
| 1. Read on-disk state.json immediately before mutation (CAS reload check)                         |
| 2. Key: <canonical_root>::<branch> (fallback: <canonical_root>)                                  |
| 3. Verify target_stage > current_stage && current_stage.can_transition_to(...)                    |
| 4. Never overwrite Manual source with Inferred source at same/lower stage                         |
| 5. Persist atomically via write_atomic (temp file + rename)                                       |
+---------------------------------------------------------------------------------------------------+
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

## 3. Concurrency Control (Reload-Before-Save Pattern)

In `src/state/state.rs`:

```rust
impl State {
    /// Mutates workflow state using read-before-write compare-and-swap reload.
    /// Reloads state.json from disk immediately before mutation to minimize race windows.
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

### Known Tradeoff & Concurrency Guarantee
- `atomic_update_workflow` eliminates stale in-memory writes spanning an entire agent turn (which may last several minutes to hours).
- The race window is reduced strictly to the filesystem IO duration of the reload and atomic rename.
- It is not a distributed lock or multi-attempt retry loop; concurrent writes within the sub-millisecond rename window remain last-writer-wins. This is an explicit architectural tradeoff to avoid process-blocking file locks.

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

### 4.3 Stage Deduction Logic
1. **Transitory Guard**: If in rebase/merge, return `None`.
2. **Feature Resolution**: If on `feat/*` or `fix/*`, derive sanitized feature name. Walk `openspec/changes/<feature>/`.
3. **OpenSpec Stages (Stages 2, 3, 4, 5)**:
   - `proposal.md` + `spec.md` without tasks -> Stage 2 (OpenSpec).
   - `tasks.md` with `total_tasks > 0` and `completed_tasks == 0` -> Stage 3 (Plan).
   - `0 < completed_tasks < total_tasks` -> Stage 4 (Work/TDD).
   - `completed_tasks == total_tasks` -> Stage 5 (Verify).
4. **Direct Entry Bypass (Stage 4)**:
   - If no OpenSpec directory, but branch is `fix/*` or `feat/*` with dirty files -> Stage 4 (Work/TDD, direct `/ce-debug` path).
5. **Knowledge Capture (Stage 6)**:
   - Modified/new files under `docs/solutions/` on the branch -> Stage 6 (Compound).
6. **Git Shipping (Stage 7)**:
   - Branch merged or open PR detected via git/gh -> Stage 7 (Ship).
7. **Monotonic Guard**:
   - Inferred stage must be strictly greater than current stage (`target > current`), legal transition (`can_transition_to`), and must never overwrite a `source == WorkflowSource::Manual` checkpoint.

---

## 5. Comprehensive Harness Hook Lifecycle

### 5.1 Claude Code (`src/harness/claude.rs`)
- In `.claude/settings.json`:
  - `hooks.SessionStart`: Turn-0 drift delivery (`ce-ai workflow resume`).
  - `hooks.Stop`: Turn-end auto-checkpoint (`ce-ai workflow resume`).
  - `hooks.PreCompact`: Pre-compaction checkpoint (`ce-ai workflow resume`).
- `remove_session_start_hook` updated to symmetrically strip `SessionStart`, `Stop`, and `PreCompact`.

### 5.2 Codex CLI (`src/harness/codex.rs`)
- In `.codex/config.toml`:
  - `hooks.SessionStart`: Turn-0 drift delivery.
  - `hooks.Stop`: Turn-end auto-checkpoint.
  - `hooks.PreCompact`: Pre-compaction checkpoint.
- `remove_session_start_hook` updated to symmetrically strip `SessionStart`, `Stop`, and `PreCompact`.

### 5.3 Cursor (`src/harness/cursor.rs`)
- In `.cursor/hooks.json`:
  - `hooks.sessionStart`: Turn-0 drift delivery (`ce-ai workflow resume --json`).
  - `hooks.stop`: Turn-end auto-checkpoint (`ce-ai workflow resume --json`).
- `remove_session_start_hook` updated to symmetrically strip `sessionStart` and `stop`.

### 5.4 GitHub Copilot CLI (`src/harness/copilot.rs`)
- In `.github/hooks/hooks.json`:
  - `hooks.sessionStart`: Turn-0 drift delivery (`ce-ai workflow resume --json`).
  - `hooks.postToolUse`: Turn-end / tool checkpoint with `additionalContext`.
- `remove_session_start_hook` updated to symmetrically strip `sessionStart` and `postToolUse`.

### 5.5 Google Antigravity (`src/harness/agy.rs`)
- In `.agents/hooks.json` under `"compound-engineering"`:
  - `"PreInvocation"`: `ce-ai workflow resume --pre-invocation`.
  - `"Stop"`: `ce-ai workflow resume` (Turn-end auto-checkpoint).
- `remove_pre_invocation_hook` updated to symmetrically strip BOTH `"PreInvocation"` and `"Stop"`.

### 5.6 Pi (`src/harness/pi.rs`)
- In `.pi/extensions/compound-engineering.ts` (`PI_EXTENSION_CONTENT`):
  - `before_agent_start`: Turn-0 context injection.
  - `agent_end`: Turn-end auto-checkpoint.
  - `session_before_compact`: Pre-compaction checkpoint.
- Version header: `// ce-ai:hook v=2`.

### 5.7 OpenCode (`src/opencode/plugins.rs`)
- In `.opencode/plugins/compound-engineering.js` (`BUILTIN_LOADER`):
  - `event.type === "session.created"`: Turn-0 context injection.
  - `event.type === "session.idle"`: Turn-end auto-checkpoint.
  - `experimental.session.compacting`: Pre-compaction checkpoint.

---

## 6. Versioning, Upgrade & Product Contract

1. **`init-prj --force`**: Checks hook completeness and version. If stale or `--force` is specified, rewrites/refreshes hook configurations.
2. **`sync` and `upgrade`**: Wire `verify_and_refresh_harness_hooks` into `ce-ai sync` and `ce-ai upgrade` to ensure existing projects automatically adopt new hooks.
3. **Product Contract (Adoption-Level Opt-In & Configurable Opt-Out)**:
   - Adopting a project via `ce-ai init-prj` is the explicit opt-in action that installs harness hooks.
   - Within an adopted project, automatic stage inference is active by default.
   - Users can opt out of automated persistence at any time via `ce-ai config set auto-checkpoint false` or flag `--no-auto-checkpoint`. When opted out, hooks run read-only advice without mutating `state.json`.
