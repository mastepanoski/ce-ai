# Exploration: Workspace Scoping of FSM Checkpoints and Gitignore Automation

## 1. Technical Context & Investigation
- `ce-ai workflow checkpoint/status/resume` enables Zero-Step Drift Recovery across turns and context compactions.
- Currently, `State` holds:
  ```rust
  pub workflow: Option<WorkflowState>,
  ```
  And `src/commands/workflow.rs` calls `State::load(&state_path)` and `state.validate_and_set_workflow(...)` without scoping by workspace.
- In Git, working across multiple repositories or worktrees (`git worktree add ...`) produces different values for `git rev-parse --show-toplevel`.
- Currently, `Context::repo_root(&self)` resolves this repository/worktree root cleanly.

## 2. Evaluated Architectural Options for Workflow Storage

### Option A: Store `workflows: BTreeMap<String, WorkflowState>` in `~/.ce-ai/state.json` (CHOSEN)
- **Mechanism**: The global state file keeps a dictionary of workflow states keyed by the canonicalized absolute path string of the workspace (`ctx.repo_root().canonicalize()`).
- **Pros**:
  - Completely local to the machine: does not touch Git working trees, does not create untracked files in the repository.
  - Ephemeral session state never leaks into git commits or merges.
  - Worktrees (`repo-worktrees/feat-x`, `repo-worktrees/feat-y`) automatically get distinct keys because their worktree roots differ.
  - Backwards-compatible: can keep `pub workflow: Option<WorkflowState>` as a fallback for existing single-workspace state files.
- **Cons**:
  - Moving or renaming a directory invalidates the key (though running a new checkpoint simply establishes a new entry for the new path).

### Option B: Store workflow in `.ce-ai.json` in each repository
- **Mechanism**: Use the existing workspace-override mechanism (`.ce-ai.json`).
- **Rejected**: `.ce-ai.json` is intended for shared repository configuration (such as model role assignments for the team). If developers commit `.ce-ai.json`, ephemeral individual progress (e.g. "task: unit test 7/12") would be committed into git and cause constant merge conflicts. If placed in `.gitignore`, `.ce-ai.json` could not be used to share team model assignments.

## 3. Gitignore Coverage for `--scope workspace`
- When `--scope workspace` is used, `compound-engineering/` is created directly under the repository root.
- Its `install-manifest.json` contains absolute workstation paths (`source.path`).
- `init_prj.rs` manages a sentinel block:
  ```gitignore
  # BEGIN CE-AI MANAGED BLOCK
  .ce-ai/skills-registry.json
  compound-engineering/
  # END CE-AI MANAGED BLOCK
  ```
- Adding `compound-engineering/` to this sentinel block ensures that any adopted project ignores workspace-scoped installs.
- Additionally, `install.rs` will proactively ensure `compound-engineering/` is present in `.gitignore` when `--scope workspace` is executed, even if `init-prj` was never called.
- `deinit-prj` strips the entire sentinel block cleanly.
