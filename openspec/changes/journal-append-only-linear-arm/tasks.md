# Tasks: Append-Only JSONL Operation Journal

- [x] 1. Core Data Structures & File Layout (`src/state/journal.rs`) [~70 LOC]
  - [x] 1.1 Define `JournalHeader` (`command`, `started_at`) with serde derive.
  - [x] 1.2 Update `Journal` struct to hold `path: PathBuf`, `file: std::fs::File`, `fail_after_writes`, `writes_seen`.
  - [x] 1.3 Keep `RecordedOp` struct unchanged.
  - [x] 1.4 Retain legacy `LegacyJournalData` struct as private for backward-compatible rollback.

- [x] 2. Append-Only `arm` and `complete` Implementation (`src/state/journal.rs`) [~60 LOC]
  - [x] 2.1 Implement initial header serialization and atomic creation in `Journal::begin`.
  - [x] 2.2 Open file handle in append mode (`std::fs::OpenOptions::new().append(true)`).
  - [x] 2.3 Refactor `Journal::arm` to serialize single-line `RecordedOp` + `\n`, write to `self.file`, and call `sync_all()`.
  - [x] 2.4 Update `Journal::complete` to drop `self.file` before calling `std::fs::remove_file`.
  - [x] 2.5 Remove deprecated `persist()` helper that re-serialized entire state.

- [x] 3. Recovery Parsing, Trailing Incomplete Line Handling & `recorded_command` (`src/state/journal.rs`) [~70 LOC]
  - [x] 3.1 Refactor recovery in `Journal::begin` to read lines, parse header, and parse ops individually.
  - [x] 3.2 Ignore trailing partial/corrupt lines when header is valid, preserving preceding complete ops.
  - [x] 3.3 Add fallback to legacy `LegacyJournalData` for pre-existing single-blob journals.
  - [x] 3.4 Optimize `recorded_command` to read and parse only the first line (`JournalHeader`).

- [x] 4. Unit & Complexity Tests (`src/state/tests/journal.rs`) [~160 LOC]
  - [x] 4.1 Adapt `begin_rolls_back_applied_ops_in_reverse` to write the JSONL format.
  - [x] 4.2 Verify `complete_removes_journal_and_content_survives`, `corrupt_journal_is_treated_as_absent`, and `fault_injection_fails_after_n_successful_arms` pass.
  - [x] 4.3 Add test for incomplete trailing line recovery (mid-write crash simulation).
  - [x] 4.4 Add test for fault injection + deterministic recovery with the new JSONL format.
  - [x] 4.5 Add quantitative linear-complexity test asserting $O(N)$ bytes written across 500+ mutations without quadratic amplification.

- [x] 5. Verification, SemVer Bump, and CHANGELOG [~30 LOC]
  - [x] 5.1 Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] 5.2 Run `cargo test --all-features` and `make e2e`.
  - [x] 5.3 Bump patch version to `1.44.2` in `Cargo.toml` and update `CHANGELOG.md`.
  - [x] 5.4 Document architectural solution in `docs/solutions/architecture/journal-append-only-linear-arm.md`.
