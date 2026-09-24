# Specification: Journal Linear Complexity Test Timing Calibration

## Requirements

### Requirement 1: Calibrated Wall-Clock Upper Bound
- **WHEN** `journal_arm_complexity_is_strictly_linear` is executed in `src/state/tests/journal.rs` across any supported CI platform (Linux, macOS, Windows)
- **THEN** the test MUST assert that 500 arming operations finish in less than 30 seconds (`elapsed.as_secs() < 30`).

### Requirement 2: Algorithmic Invariants Preserved
- **WHEN** 500 arm operations are performed sequentially
- **THEN** every single write operation MUST increase file size by less than 3,000 bytes (`max_delta < 3_000`).
- **AND** the final journal file size MUST not exceed 1,500,000 bytes (`final_size < 1_500_000`).
- **AND** total bytes appended MUST exactly match the file size change.

### Requirement 3: SemVer & Release Compliance
- **WHEN** the fix is prepared for shipping
- **THEN** `Cargo.toml` MUST be bumped to `1.68.1`.
- **AND** `CHANGELOG.md` MUST record the fix under `## [1.68.1] - 2026-09-23`.
