# Proposal: Fix Journal Linear Complexity Test Flaky Wall-Clock Assertion on Windows CI

## Problem Statement
In `src/state/tests/journal.rs`, the unit test `journal_arm_complexity_is_strictly_linear` was introduced to prevent regressions back to quadratic $O(N^2)$ journal serialization, which previously caused mutation arming to take upwards of 10 minutes (> 600s).

The test iterates 500 times over `j.arm(path)`. In addition to mathematical invariants verifying $O(1)$ write amplification (`max_delta < 3_000`) and linear journal size (`final_size < 1_500_000`), the test asserts a hard wall-clock threshold:
```rust
assert!(
    elapsed.as_secs() < 10,
    "500 arm() calls took {:?}, expected < 10s for linear complexity",
    elapsed
);
```

Each `arm()` call executes `self.file.sync_all()`, which maps to `FlushFileBuffers` on Windows NTFS. On virtualized GitHub Actions Windows runners (`windows-latest`), shared storage I/O contention occasionally yields ~20ms per fsync call, causing 500 operations to take ~10.05 seconds. This tripped the hard `< 10` second assertion post-merge (run #35783770869), despite the algorithmic behavior being completely linear and all functional checks passing.

## In-Scope
1. **Wall-Clock Threshold Calibration**:
   - Relax the wall-clock assertion in `src/state/tests/journal.rs` from `< 10` to `< 30` seconds.
   - 30 seconds provides ample margin (3x) for Windows NTFS runner I/O jitter while remaining 20x faster than the 10-minute quadratic regression it guards against.
2. **Mathematical Invariant Preservation**:
   - Retain all deterministic mathematical assertions: `max_delta < 3_000` (proving $O(1)$ write amplification per operation), `final_size < 1_500_000` (proving $O(N)$ journal footprint), and `total_appended == 0`.
3. **Documentation and Explanatory Comments**:
   - Update the code comments in `src/state/tests/journal.rs` to clearly document the platform I/O variance on Windows NTFS runners and the rationale for the 30-second bound.
4. **SemVer & Release Hygiene**:
   - Bump SemVer patch to `v1.68.1` in `Cargo.toml`.
   - Document the fix in `CHANGELOG.md`.

## Out-of-Scope
- Removing `self.file.sync_all()` from `arm()`: synchronous durability on mutation arming is an essential crash-recovery invariant and must never be compromised.
- Modifying journal file format or parsing logic.

## Risk Evaluation & Mitigation
- **Risk (Regressing to Quadratic Serialization)**: Will relaxing to `< 30s` allow a quadratic serialization bug to slip through?
  - *Mitigation*: No. The quadratic regression takes > 600 seconds (10 minutes) for 500 operations. Furthermore, assertion #1 (`max_delta < 3_000`) and assertion #2 (`final_size < 1_500_000`) are purely mathematical and fail immediately if quadratic serialization occurs, independent of elapsed wall time.
