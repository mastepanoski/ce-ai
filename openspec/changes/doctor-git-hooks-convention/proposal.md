# Proposal: Condition Doctor Git-Hooks Probe on Project Adoption of .githooks Convention

## Problem Statement
The `ce-ai doctor` command includes a git-hooks verification probe (`src/commands/doctor.rs:297-333`) originally created as a self-check for `ce-ai`'s own repository convention (guarding against git worktree contention resetting `core.hooksPath`). However, because it runs unconditionally across all adopted projects, it creates false-positive findings whenever a project legitimately uses an alternative git hooks manager such as Husky (`.husky/_`), lefthook, or pre-commit.

When `core.hooksPath` points to anything other than `.githooks`, `doctor` emits:
`git-hooks: core.hooksPath set to '<val>', expected '.githooks'`
This finding increments `findings.len()`, causing `ce-ai doctor` to fail with exit code 1 (or exit code 2 when `--strict` is enabled), despite the user's repository having a perfectly valid, non-drifted hooks configuration.

## Boundaries

### In Scope
- Conditioning the `.githooks` enforcement in `src/commands/doctor.rs` on whether the project has actually adopted the `.githooks` convention (i.e. whether `<root_path>/.githooks/` directory exists).
- If `core.hooksPath` points to `.githooks`, verify that `.githooks/pre-commit` exists (finding if missing).
- If `core.hooksPath` points elsewhere AND `<root_path>/.githooks/` exists, report a drift finding (`git-hooks: core.hooksPath set to '...', expected '.githooks'`).
- If `core.hooksPath` points elsewhere AND `<root_path>/.githooks/` does NOT exist, log `doctor-info` (project uses a different hooks manager; skip without finding).
- Updating existing test in `tests/cli.rs` (`doctor_reports_git_hooks_misconfigured_finding`) to create `.githooks` directory so genuine drift continues to be validated.
- Adding regression test in `tests/cli.rs` (`doctor_ignores_non_githooks_hooks_path_when_not_adopted`) to verify that non-`.githooks` hooks paths (such as `.husky/_`) are not flagged when `.githooks/` is absent.

### Out of Scope
- Auditing or managing third-party hook configurations (Husky, lefthook, pre-commit frameworks).
- Changes to `ce-ai init-prj` or adoption hooks.
- MCP tools detection (tracked in Issue #293).

## Risk Evaluation
- **Drift Regression Risk**: Projects that deliberately use `.githooks` must still be protected against accidental drift caused by worktree creation or git config resetting `core.hooksPath`. By checking `root_path.join(".githooks").exists()`, any project with `.githooks/` present will strictly retain this drift protection.

## Success Criteria
- Projects with alternative git hook managers (e.g. `core.hooksPath = .husky/_`) and no `.githooks/` dir produce zero findings from the git-hooks probe and output informational log `doctor-info: git-hooks core.hooksPath set to '...' (not the .githooks convention; skipping)`.
- Projects that have adopted `.githooks/` continue to report drift findings if `core.hooksPath` points elsewhere or if `.githooks/pre-commit` is missing.
- 100% test pass rate across `cargo test`.
