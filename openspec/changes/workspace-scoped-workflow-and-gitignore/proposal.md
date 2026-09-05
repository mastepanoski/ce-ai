# Proposal: Workspace-Scoped Workflow FSM and Workspace-Scope `.gitignore` Coverage

## 1. Problem Statement
1. **Global Workflow Collision in FSM**:
   `State.workflow: Option<WorkflowState>` in `src/state/state.rs` stores a single global workflow checkpoint in `~/.ce-ai/state.json`. When developers work across multiple repositories or multiple git worktrees on the same machine (a workflow pattern explicitly recommended in `docs/user-guide/quick-start-workflow-guide.md` section "Working with Multiple Git Worktrees"), running `ce-ai workflow checkpoint` in one workspace silently overwrites and destroys the active FSM stage, task, and feature context of all other workspaces.
2. **Missing `.gitignore` Coverage for `--scope workspace` Artifacts**:
   `ce-ai init-prj` injects a sentinel-bounded `.gitignore` block containing only `.ce-ai/skills-registry.json`. When a user runs `ce-ai install --harness opencode --scope workspace` (recommended for worktree isolation), `install.rs` generates `<workspace>/compound-engineering/install-manifest.json`, which records local absolute paths in its `source` metadata. Without `compound-engineering/` in `.gitignore`, this directory is left untracked and vulnerable to inadvertent `git add -A` commits, leaking workstation-specific absolute paths into shared repositories.

## 2. Scope & Boundaries

### In Scope
- Scope `State` workflow tracking per workspace root path (canonicalized) in `~/.ce-ai/state.json` via a `workflows: BTreeMap<String, WorkflowState>` map.
- Maintain full backwards compatibility with legacy `state.json` containing the scalar `workflow: Option<WorkflowState>` field.
- Update `ce-ai workflow checkpoint`, `status`, and `resume` (both CLI text output and `--json` format) to query and update the workflow state for the active `repo_root`.
- Update `init-prj` to include `compound-engineering/` in its sentinel-bounded `.gitignore` block.
- Update `install --scope workspace` to ensure `compound-engineering/` is ignored in `.gitignore` even if `init-prj` was not run beforehand.
- Update `deinit-prj` to cleanly strip the managed sentinel block.
- Update user documentation in `quick-start-workflow-guide.md`, `zero-step-drift-recovery-explained.md`, and `installation-and-coexistence-mechanisms.md`.
- Comprehensive unit and CLI integration tests validating cross-workspace isolation and `.gitignore` automation.

### Out of Scope
- Committing workflow state to `.ce-ai.json` (workflow state is ephemeral session progress and MUST remain local to the machine's `~/.ce-ai/state.json`).
- Heavy external file-locking dependencies like `fs2` (we document last-writer-wins considerations and keep atomic writes).

## 3. Success Criteria
- [x] Checkpoint in Workspace A followed by Checkpoint in Workspace B leaves Workspace A's checkpoint intact when executing `ce-ai workflow status` or `resume` from Workspace A.
- [x] Legacy `state.json` files with top-level scalar `workflow` deserialize and resolve seamlessly.
- [x] `init-prj` writes `compound-engineering/` into the sentinel block of `.gitignore`.
- [x] `install --scope workspace` ensures `compound-engineering/` is in `.gitignore`.
- [x] 100% passing tests, clippy clean, fmt clean, and green CI matrix.
