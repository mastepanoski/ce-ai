# Proposal: CodeGraph Subprocess Execution Happy Path Coverage

## 1. Problem Statement
In `ce-ai v1.39.0`, native CodeGraph index initialization support was introduced via `ce-ai tools init codegraph` and auto-initialization in `ce-ai init-prj`. While error handling (missing binary, unsupported tools), idempotency (already-initialized index), and `--dry-run` modes were thoroughly tested, there was a coverage gap:
- No test exercised the end-to-end happy path where `codegraph` executable exists on `PATH` and executes `codegraph init` with exit code `0`, creating the `.codegraph/` directory on disk.
- Relying on the host machine's installed `codegraph` binary is non-hermetic and fails in CI environments where `codegraph` is not installed on PATH.

## 2. In-Scope / Out-of-Scope
- **In-Scope**:
  - Implement a hermetic mock executable helper (`fake_codegraph`) in `tests/cli.rs` following the existing `fake_gh` pattern.
  - Test the full happy path of `ce-ai tools init codegraph` (verifying `.codegraph/` creation and success output).
  - Test the auto-initialization happy path of `init_codegraph_if_available` during `ce-ai init-prj`.
  - Bump SemVer to `1.39.1` and document in `CHANGELOG.md`.
- **Out-of-Scope**:
  - Modifying the production logic of `src/commands/tools.rs` or `src/commands/init_prj.rs` (which already work correctly).

## 3. Risk Evaluation
- **Cross-Platform Compatibility**: Creating mock scripts with shell shebangs requires `#[cfg(unix)]` or cross-platform batch handling. Following the established `#[cfg(unix)]` pattern from `fake_gh` ensures full reliability on Unix and macOS without breaking Windows CI runners.

## 4. Success Criteria
- Automated integration test verifies `ce-ai tools init codegraph` creates `.codegraph/` and reports `✓ Initialized CodeGraph index`.
- Automated integration test verifies `ce-ai init-prj` creates `.codegraph/` when `codegraph` is present on `PATH`.
- 100% green CI matrix across Linux, macOS, and Windows.
