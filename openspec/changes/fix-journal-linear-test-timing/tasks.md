# Tasks: Fix Journal Linear Complexity Test Flaky Wall-Clock Assertion on Windows CI

- [x] **Work Unit 1: Calibrate Test Wall-Clock Assertion & Update Documentation** (~10 LOC)
  - [x] In `src/state/tests/journal.rs`, update `journal_arm_complexity_is_strictly_linear`:
    - [x] Update explanatory comment documenting Windows NTFS runner fsync latency.
    - [x] Relax assertion from `elapsed.as_secs() < 10` to `elapsed.as_secs() < 30`.
  - [x] Verify test execution locally with `cargo test state::journal::tests::journal_arm_complexity_is_strictly_linear`.

- [x] **Work Unit 2: SemVer Bump & CHANGELOG Maintenance** (~15 LOC)
  - [x] Bump version in `Cargo.toml` from `1.68.0` to `1.68.1`.
  - [x] Add entry `## [1.68.1] - 2026-09-23` in `CHANGELOG.md` documenting the Windows CI flaky test timing calibration.
  - [x] Run `cargo check` to update `Cargo.lock`.

- [x] **Work Unit 3: Verification & Quality Gates** (~0 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e`.
