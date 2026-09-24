# Design: Calibrated Wall-Clock Assertion for Journal Linear Complexity Test

## Architecture & Contract Overview

The test `src/state/tests/journal.rs::journal_arm_complexity_is_strictly_linear` validates that appending mutation records to the crash-recovery journal executes in linear time $O(N)$ with constant space $O(1)$ write amplification.

### Test Implementation Design

```rust
// 4. Wall time for 500 arms with 500 physical fsyncs must finish well within 30s
// (on macOS APFS / Linux runners, 500 fsyncs take ~1-3s in debug build;
// on Windows NTFS CI runners, fsync/FlushFileBuffers can take ~8-12s under I/O load;
// the previous quadratic serialization took >10 minutes).
assert!(
    elapsed.as_secs() < 30,
    "500 arm() calls took {:?}, expected < 30s for linear complexity (quadratic took >10m)",
    elapsed
);
```

### Safety and Invariant Properties

1. **Deterministic Complexity Invariants**:
   - $\Delta \text{size}_i < 3000$ bytes for each $i \in [1, 500]$: Guarantees no serialized payload aggregation during individual `arm()` writes.
   - $\text{Total Size} < 1,500,000$ bytes: Guarantees total storage is strictly $O(N)$ (actual ~950 KB for 500 files).
2. **Empirical Wall-Clock Boundary**:
   - Old bug: $O(N^2)$ serialization took $> 600$s ($> 10$m).
   - Calibrated bound: $< 30$s ensures that any quadratic regression is caught by a factor of 20x.
   - Runner tolerance: Accommodates Windows NTFS runner latency (up to 60ms per fsync).
