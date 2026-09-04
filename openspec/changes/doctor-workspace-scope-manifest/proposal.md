# Proposal: Workspace-Scoped OpenCode Manifest Resolution in Doctor, Status, and Sync

## Problem Statement
When `ce-ai install --harness opencode --scope workspace` is executed (as recommended for isolated Git worktrees in `docs/user-guide/quick-start-workflow-guide.md`), `install-manifest.json` and OpenCode plugin assets are written to the workspace root (`<repo_root>/compound-engineering/`). However, the global `state.json` under `ctx.config_dir` records the `"opencode"` entry without preserving the installation scope or target directory, and `ce-ai doctor`, `status`, and `sync` unconditionally probe `ctx.opencode_config_dir` (`~/.config/opencode`).

Consequently, running `ce-ai doctor` in a workspace-scoped environment immediately fails with exit code 1 due to two false positive findings:
1. `state-inconsistent: opencode state entry and install manifest disagree`
2. `opencode: SessionStart plugin missing or outdated in '~/.config/opencode'`

Furthermore, `ce-ai status` reports `drift: unknown (no install manifest)` and `ce-ai sync` errors with `no install-manifest.json — run install first`.

## Boundaries

### In Scope
- Storing explicit `"scope"` (`"workspace"` or `"global"`) and `"target_dir"` in `state.installed_harnesses` during `ce-ai install`.
- Implementing a helper on `Context` (`resolve_opencode_dir(&self, state: &State) -> PathBuf`) that resolves the active OpenCode directory:
  - If the active workspace matches a workspace-scoped `"opencode"` entry, or if the current workspace root contains `<repo_root>/compound-engineering/install-manifest.json`, resolve to the workspace root.
  - Otherwise, resolve to `ctx.opencode_config_dir`.
- Updating `ce-ai doctor` to evaluate configuration validity, manifest existence, drift, and the SessionStart plugin against the resolved OpenCode directory.
- Updating `ce-ai status` and `ce-ai sync` to evaluate and update drift/manifest against the resolved OpenCode directory.
- Integration test in `tests/cli.rs` and unit test in `src/commands/tests/doctor.rs`.

### Out of Scope
- Redesigning multi-harness registry for non-OpenCode harnesses (Claude, Cursor, Codex, Pi already have native directories or worktree rules).
- Modifying project adoption rules (`init-prj`).

## Risk Evaluation
- **Backward Compatibility**: Existing installations may have `state.installed_harnesses` entries without a `"scope"` field. The resolver must fall back to checking if `<workspace_root>/compound-engineering/install-manifest.json` exists before falling back to `ctx.opencode_config_dir`.
- **False Negative Risk**: A real inconsistency (e.g. manifest deleted by the user) in either global or workspace scope must still trigger `state-inconsistent`.

## Success Criteria
- `ce-ai install --harness opencode --scope workspace --source <fixture>` followed by `ce-ai doctor` reports 0 findings and exits with code 0.
- `ce-ai status` in a workspace-scoped install reports accurate drift (`drift: none`).
- `ce-ai sync` in a workspace-scoped install reconciles against the workspace manifest.
- All existing unit, CLI integration, and security tests pass green.
