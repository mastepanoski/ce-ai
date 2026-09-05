# Tasks: CodeGraph Subprocess Execution Happy Path Coverage

Total Estimated Changed Lines: ~75 LOC (Forecast: well within 400 LOC budget).

- [ ] **Task 1: Add `fake_codegraph` helper in `tests/cli.rs`** (~25 LOC)
  - Create shell script mock returning version `v1.4.1` on `--version` and creating `.codegraph/` on `init`.
  - Set 0o755 executable permissions under `#[cfg(unix)]`.

- [ ] **Task 2: Add integration tests in `tests/cli.rs`** (~40 LOC)
  - `tools_init_codegraph_happy_path_creates_index`: tests `ce-ai tools init codegraph` end-to-end with mock binary.
  - `init_prj_auto_initializes_codegraph_when_present`: tests `ce-ai init-prj` invoking `init_codegraph_if_available` with mock binary.

- [ ] **Task 3: Versioning and Documentation** (~10 LOC)
  - Bump SemVer to `1.39.1` in `Cargo.toml` and `Cargo.lock`.
  - Add `1.39.1` entry in `CHANGELOG.md`.
