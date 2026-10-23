# Tasks: Deterministic Tasks Tail Validator

- [x] **Unit 1 (~150 LOC):** Implementation of `scripts/validate-tasks-tail.py`
  - Implement command line parser with flags: `--fix`, `--check-code`, `--allow-exempt`, `--json`, and positional `path`.
  - Implement heuristic for detecting code changes vs docs-only.
  - Implement tail scanning logic for `simplify`, `review`, and `compound`.
  - Implement idempotent `--fix` appender.
  - *Verification*: Manual test on sample `tasks.md` files.

- [x] **Unit 2 (~120 LOC):** Automated test suite for validator
  - Create `tests/test_validate_tasks_tail.py` using standard `unittest`.
  - Cover missing tail failure, passing tail, `--fix` idempotence, and docs exemption.
  - *Verification*: `python3 tests/test_validate_tasks_tail.py` passes 100%.

- [x] **Unit 3: Code Simplification & Refactoring (`ce-simplify-code`)**
  - Audit `scripts/validate-tasks-tail.py` for clarity, reuse, and robust error handling.
  - Ensure compatibility with Windows paths and UNIX paths.

- [x] **Unit 4: Structured Code Review & Review Receipt**
  - Run verification checks and record review receipt: `cargo run -- workflow review-receipt`.

- [x] **Unit 5: Knowledge Compounding & Vocabulary Capture (`ce-compound`)**
  - Create solution artifact `docs/solutions/developer-tooling/deterministic-tasks-lifecycle-validator.md`.
  - Monotonically accrete `CONCEPTS.md` with *Tasks Lifecycle Tail*.
  - *Verification*: `cargo run -- doc lint --strict`.

- [x] **Unit 6: Verification, SemVer bump, and PR creation**
  - Bump PATCH version in `Cargo.toml` (`1.74.2`) and add entry to `CHANGELOG.md`.
  - Run full test gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`, `cargo run -- doc lint --strict`.
  - Open PR with self-explaining description and empirical evidence.
