# Proposal: Hermetic Git Environment in Test Fixtures

## 1. Problem Statement
In Issue #302, 3 tests in `tests/cli.rs` were observed failing intermittently during local pre-commit hook executions (`.githooks/pre-commit`):
- `audit_suggests_codegraph_init_without_gentle_ai`
- `doctor_workspace_scope_opencode_install_has_no_false_positive_findings`
- `install_workspace_scope_ensures_compound_engineering_in_gitignore`

Investigation revealed this was not a timing race or parallel load issue, but a deterministic environment pollution bug:
- When tests execute inside a Git hook (e.g., pre-commit), Git automatically sets `GIT_DIR`, `GIT_INDEX_FILE`, `GIT_PREFIX`, and `GIT_WORK_TREE` in the process environment.
- While the test runner helper `ceai(...)` stripped these variables from child `ce-ai` commands, the test fixtures themselves invoked `std::process::Command::new("git")` directly during repository setup without stripping `GIT_DIR`.
- Consequently, `git init` within the test fixture did not initialize a local `.git` inside the temporary fixture directory, but instead targeted the parent checkout. When `ce-ai` subsequently executed `git rev-parse --show-toplevel` (with `GIT_DIR` stripped), it failed to detect a git repository, breaking workspace scoping and repo-level audit checks.

## 2. In-Scope / Out-of-Scope Boundaries
- **In-Scope**:
  - Introduce a centralized `git_cmd()` helper in `tests/cli.rs` that returns a `std::process::Command` pre-configured to strip `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_PREFIX`.
  - Replace all raw `std::process::Command::new("git")` instantiations in `tests/cli.rs` with `git_cmd()`.
  - Add a dedicated integration test verifying that tests run cleanly even when `GIT_DIR` and `GIT_INDEX_FILE` are set in the outer process environment.
  - Bump SemVer to `1.39.3` in `Cargo.toml` and document in `CHANGELOG.md`.
- **Out-of-Scope**:
  - Changing production CLI code in `src/commands/audit.rs`, `src/commands/install.rs`, or `src/commands/doctor.rs` (which behave correctly).

## 3. Risk Evaluation
- **Zero Production Risk**: Changes are strictly confined to test fixture helpers in `tests/cli.rs`.
- **Hermetic Guarantee**: Eliminates pre-commit hook flakes and developer friction permanently across all platforms.

## 4. Success Criteria
- Running `GIT_DIR=... GIT_INDEX_FILE=... cargo test --test cli` passes 100% (148/148 tests green).
- Full CI matrix passes green across Linux, macOS, and Windows.
