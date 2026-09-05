# Specification: Hermetic Git Environment in Test Fixtures

## Requirements

### REQ-1: Hermetic Git Fixture Helper
- **WHEN** test functions in `tests/cli.rs` execute git commands to construct or inspect repositories,
- **THEN** the execution SHALL use `git_cmd()` with `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_PREFIX` removed from the command's environment.

### REQ-2: Clean Pre-Commit Execution
- **WHEN** `cargo test --test cli` runs within an environment where `GIT_DIR` and `GIT_INDEX_FILE` are exported (such as git pre-commit hooks),
- **THEN** all tests SHALL pass without failure or false positive state-inconsistent reports.

## Acceptance Criteria
1. `audit_suggests_codegraph_init_without_gentle_ai` passes when `GIT_DIR` is set.
2. `doctor_workspace_scope_opencode_install_has_no_false_positive_findings` passes when `GIT_DIR` is set.
3. `install_workspace_scope_ensures_compound_engineering_in_gitignore` passes when `GIT_DIR` is set.
4. Full test suite passes 100% (148/148) under simulated pre-commit environment.
