# Tasks: Append-Only JSONL Operation Journal

- [ ] 1. Core Data Structures & File Layout (`src/state/journal.rs`) [~70 LOC]
  - [ ] 1.1 Define `JournalHeader` (`command`, `started_at`) with serde derive.
  - [ ] 1.2 Update `Journal` struct to hold `path: PathBuf`, `file: std::fs::File`, `command: String`, `fail_after_writes`, `writes_seen`.
  - [ ] 1.3 Keep `RecordedOp` struct unchanged.
  - [ ] 1.4 Retain legacy `JournalData` struct as private for backward-compatible rollback.

- [ ] 2. Append-Only `arm` and `complete` Implementation (`src/state/journal.rs`) [~60 LOC]
  - [ ] 2.1 Implement initial header serialization and atomic creation in `Journal::begin`.
  - [ ] 2.2 Open file handle in append mode (`std::fs::OpenOptions::new().append(true)`).
  - [ ] 2.3 Refactor `Journal::arm` to serialize single-line `RecordedOp` + `\n`, write to `self.file`, and call `sync_all()`.
  - [ ] 2.4 Update `Journal::complete` to drop `self.file` before calling `std::fs::remove_file`.
  - [ ] 2.5 Remove deprecated `persist()` helper that re-serialized entire state.

- [ ] 3. Recovery Parsing, Trailing Incomplete Line Handling & `recorded_command` (`src/state/journal.rs`) [~70 LOC]
  - [ ] 3.1 Refactor recovery in `Journal::begin` to read lines, parse header, and parse ops individually.
  - [ ] 3.2 Ignore trailing partial/corrupt lines when header is valid, preserving preceding complete ops.
  - [ ] 3.3 Add fallback to legacy `JournalData` for pre-existing single-blob journals.
  - [ ] 3.4 Optimize `recorded_command` to read and parse only the first line (`JournalHeader`).

- [ ] 4. Unit & Complexity Tests (`src/state/tests/journal.rs`) [~160 LOC]
  - [ ] 4.1 Adapt `begin_rolls_back_applied_ops_in_reverse` to write the JSONL format.
  - [ ] 4.2 Verify `complete_removes_journal_and_content_survives`, `corrupt_journal_is_treated_as_absent`, and `fault_injection_fails_after_n_successful_arms` pass.
  - [ ] 4.3 Add test for incomplete trailing line recovery (mid-write crash simulation).
  - [ ] 4.4 Add test for fault injection + deterministic recovery with the new JSONL format.
  - [ ] 4.5 Add quantitative linear-complexity test asserting $O(N)$ bytes written across 500+ mutations without quadratic amplification.

- [ ] 5. Verification, SemVer Bump, and CHANGELOG [~30 LOC]
  - [ ] 5.1 Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
  - [ ] 5.2 Run `cargo test --all-features` and `make e2e`.
  - [ ] 5.3 Bump patch version to `1.44.2` in `Cargo.toml` and update `CHANGELOG.md`.
  - [ ] 5.4 Document architectural solution in `docs/solutions/architecture/journal-append-only-linear-arm.md`.
