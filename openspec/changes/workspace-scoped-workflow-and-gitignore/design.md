# Design: Workspace Scoping of FSM Checkpoints and Gitignore Automation

## 1. Schema Extensions in `src/state/state.rs`

### Struct Extensions
```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    // ... other existing fields ...

    /// Legacy scalar workflow field, preserved for backwards compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowState>,

    /// Workspaces map keyed by canonical workspace root string.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workflows: BTreeMap<String, WorkflowState>,

    // ... other existing fields ...
}
```

### Methods in `State`
1. `normalize_workspace_key(root: &Path) -> String`:
   Canonicalizes `root` if possible via `std::fs::canonicalize(root)`, falling back to `root.to_string_lossy().to_string()`.
2. `current_workflow_for(&self, root: &Path) -> Option<WorkflowState>`:
   - Key lookup: checks `self.workflows.get(&normalize_workspace_key(root))`.
   - Fallback: if not found, falls back to `self.workflow.clone()`.
3. `validate_and_set_workflow_for(&mut self, root: &Path, target_stage: WorkflowStage, task: &str, feature: Option<String>) -> Result<(), CeError>`:
   - Queries previous state via `self.current_workflow_for(root)`.
   - Validates transition with `can_transition_to`.
   - Computes feature inheritance / clearing for Stage 1 reset.
   - Inserts the new `WorkflowState` into `self.workflows.insert(normalize_workspace_key(root), new_wf)`.
   - Updates `self.workflow = Some(new_wf)` so legacy tooling reading `state.workflow` continues to see the latest workflow.
4. Preserves `current_workflow(&self)` and `validate_and_set_workflow(&mut self, ...)` as delegates using `std::env::current_dir()`.

## 2. Command Workflow in `src/commands/workflow.rs`
- In `checkpoint_lines`:
  Resolves `ws_root = ctx.repo_root()`. Calls `state.validate_and_set_workflow_for(&ws_root, stage, task, feature.map(String::from))`.
- In `status_lines`:
  Resolves `ws_root = ctx.repo_root()`. Calls `state.current_workflow_for(&ws_root)`.
- In `resume_lines`:
  Resolves `ws_root = ctx.repo_root()`. Calls `state.current_workflow_for(&ws_root)`.
- In `run` (Action::Status { json }, Action::Checkpoint { json }, Action::Resume { json }):
  Uses `state.current_workflow_for(&ctx.repo_root())`.

## 3. Gitignore Management in `init_prj.rs` and `install.rs`
- In `src/commands/init_prj.rs`:
  ```rust
  let gitignore_block = format!(
      "{}\n.ce-ai/skills-registry.json\ncompound-engineering/\n{}\n",
      GITIGNORE_BEGIN_MARKER, GITIGNORE_END_MARKER
  );
  ```
- In `src/commands/install.rs`:
  When `scope_arg == "workspace"`, check `<repo>/.gitignore`. If it does not contain `"compound-engineering/"`, append it to the file atomically or inside the sentinel block.
- In `src/commands/deinit_prj.rs`:
  The existing logic already removes everything between `GITIGNORE_BEGIN_MARKER` and `GITIGNORE_END_MARKER`, which will cleanly remove the entire updated block.
