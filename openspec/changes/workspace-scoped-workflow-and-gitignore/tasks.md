# Tasks: Workspace-Scoped Workflow FSM and Gitignore Automation

- [x] Unit 1: State Schema & Scoped Workflow API in `src/state/state.rs` (~75 LOC)
  - [x] Add `workflows: BTreeMap<String, WorkflowState>` to `State` struct with serde defaults and skip if empty.
  - [x] Implement `normalize_workspace_key(root: &Path) -> String`.
  - [x] Implement `current_workflow_for(&self, root: &Path) -> Option<WorkflowState>` with fallback to legacy `workflow`.
  - [x] Implement `validate_and_set_workflow_for(&mut self, root: &Path, stage: WorkflowStage, task: &str, feature: Option<String>) -> Result<(), CeError>`.
  - [x] Update `current_workflow(&self)` and `validate_and_set_workflow(&mut self, ...)` to delegate seamlessly.

- [x] Unit 2: Workflow Commands Scoping in `src/commands/workflow.rs` (~70 LOC)
  - [x] Update `status_lines(ctx)` to use `state.current_workflow_for(&ctx.repo_root())`.
  - [x] Update `checkpoint_lines(ctx, ...)` to use `state.validate_and_set_workflow_for(&ctx.repo_root(), ...)`.
  - [x] Update `resume_lines(ctx)` to resolve workspace root.
  - [x] Update `run` JSON serialization for `status`, `checkpoint`, and `resume` to resolve using `ctx.repo_root()`.

- [x] Unit 3: Sentinel `.gitignore` Updates in `init_prj.rs` and `install.rs` (~45 LOC)
  - [x] Update `init_prj.rs` sentinel block to include `compound-engineering/`.
  - [x] Update `install.rs` to ensure `compound-engineering/` is ignored when `--scope workspace` is used.
  - [x] Verify `deinit_prj.rs` cleanly strips the sentinel block.

- [x] Unit 4: Tests and Documentation (~180 LOC)
  - [x] Unit tests in `src/state/tests/state.rs` for `workflows` map isolation, fallback to legacy `workflow`, and deserialization compatibility.
  - [x] CLI integration tests in `tests/cli.rs`: multi-repo isolation test for workflow checkpoints, and `.gitignore` verification tests.
  - [x] Update documentation: `docs/user-guide/quick-start-workflow-guide.md`, `docs/user-guide/zero-step-drift-recovery-explained.md`, `docs/user-guide/installation-and-coexistence-mechanisms.md`, and `CHANGELOG.md`.
