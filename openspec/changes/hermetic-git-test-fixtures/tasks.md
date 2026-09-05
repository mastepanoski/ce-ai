# Tasks: Hermetic Git Environment in Test Fixtures

Total Estimated Changed Lines: ~60 LOC (Forecast: well within 400 LOC budget).

- [x] **Task 1: Add `git_cmd()` helper in `tests/cli.rs`** (~15 LOC)
  - Define `fn git_cmd() -> std::process::Command` that removes `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, and `GIT_PREFIX`.

- [x] **Task 2: Replace raw `Command::new("git")` with `git_cmd()` across `tests/cli.rs`** (~35 LOC)
  - Replace all raw `std::process::Command::new("git")` calls in test setup fixtures.

- [x] **Task 3: Version Bump & Release Documentation** (~10 LOC)
  - Bump SemVer to `1.39.3` in `Cargo.toml` and `Cargo.lock`.
  - Add `1.39.3` entry in `CHANGELOG.md` referencing Issue #302.
