# Specification: Workspace Scoping of FSM Checkpoints and Gitignore Automation

## Feature 1: Workspace-Scoped Workflow FSM Checkpoint & Resume

- **WHEN** `ce-ai workflow checkpoint` is executed in a repository or worktree `A`,
- **THEN** `ce-ai` MUST save the `WorkflowState` under the workspace key corresponding to `A` in `state.workflows` within `~/.ce-ai/state.json`.

- **WHEN** `ce-ai workflow checkpoint` is subsequently executed in another repository or worktree `B`,
- **THEN** the workflow state for `A` MUST remain unchanged in `state.workflows`.

- **WHEN** `ce-ai workflow status` or `ce-ai workflow resume` is executed in repository or worktree `A`,
- **THEN** `ce-ai` MUST retrieve and display the workflow state corresponding to `A`.

- **WHEN** loading a legacy `state.json` containing only the top-level `workflow: Option<WorkflowState>` field,
- **THEN** `ce-ai` MUST successfully deserialize the file and use `workflow` as a fallback when `workflows` does not have an entry for the active workspace.

## Feature 2: Workspace-Scoped Gitignore Automation

- **WHEN** `ce-ai init-prj` is executed in a project,
- **THEN** `ce-ai` MUST inject a sentinel-bounded block in `.gitignore` containing both `.ce-ai/skills-registry.json` and `compound-engineering/`.

- **WHEN** `ce-ai install --scope workspace` is executed,
- **THEN** `ce-ai` MUST ensure that `compound-engineering/` is ignored in the workspace's `.gitignore`.

- **WHEN** `ce-ai deinit-prj` is executed in an adopted project,
- **THEN** `ce-ai` MUST strip the sentinel-bounded block from `.gitignore`, leaving other rules intact.
