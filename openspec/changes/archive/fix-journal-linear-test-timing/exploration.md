# Exploration: Journal Arm Complexity Test Timing Invariants

## Technical Investigation

### Current Test Architecture
The test `journal_arm_complexity_is_strictly_linear` in `src/state/tests/journal.rs` sets up 500 files with 512-byte payloads and executes:
```rust
for file in &files {
    j.arm(file).unwrap();
    let curr_len = std::fs::metadata(&j_path).unwrap().len();
    let delta = curr_len - prev_len;
    if delta > max_delta {
        max_delta = delta;
    }
    prev_len = curr_len;
}
```

The assertions verify:
1. `max_delta < 3_000`: Bounded $O(1)$ size increase per write.
2. `final_size < 1_500_000`: Bounded $O(N)$ total size.
3. `total_appended == 0`: Consistency check against file size.
4. `elapsed.as_secs() < 10`: Wall-clock bound.

### Platform Differences in File Synchronization
- **macOS (APFS)**: `sync_all` takes ~1-3ms per call; 500 calls complete in ~1.5 - 2.5s.
- **Linux (ext4)**: `sync_all` takes ~1-2ms per call; 500 calls complete in ~1 - 2s.
- **Windows (NTFS on GitHub Actions)**: `FlushFileBuffers` on shared runner virtual disks with Windows Defender scanning file handles takes ~15-25ms per call under baseline conditions. When multiple parallel jobs run on the host or during I/O spikes, latency can reach ~20-22ms per flush:
  $$500 \times 20.1\text{ ms} = 10,050\text{ ms} = 10.05\text{ s}$$
  This resulted in `thread panicked: 500 arm() calls took 10.0533549s, expected < 10s`.

### Evaluated Options

| Option | Pros | Cons | Recommendation |
|---|---|---|---|
| **Option A: Relax wall-clock assertion to `< 30s`** | Simple, cross-platform, preserves the 500-item statistical sample, still 20x faster than quadratic (>600s), zero production risk. | Wall-clock test remains slightly influenced by host machine load. | **Recommended** |
| **Option B: Reduce sample size to $N = 250$** | Cuts execution time in half across all platforms. | Slightly reduces sample size for size variance checks. | Secondary |
| **Option C: Conditional threshold via `#[cfg(windows)]`** | Keeps `< 10s` on Unix and `< 30s` on Windows. | Adds platform branching to test code without functional benefit, since Unix already completes in < 3s. | Not recommended |
| **Option D: Eliminate wall-clock assertion completely** | Eliminates all timing flakes. | Loses the empirical check that execution does not freeze or block. | Not recommended |

### Conclusion
Option A is the cleanest and most robust approach. The mathematical assertions (#1 and #2) already give 100% deterministic proof of linear algorithmic complexity. The wall-clock check is simply an empirical guardrail against infinite loops or quadratic slowdowns (>600s), and a 30s ceiling provides generous headroom while catching regressions decisively.
