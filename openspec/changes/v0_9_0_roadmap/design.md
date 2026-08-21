# OpenSpec Design: Release v0.9.0 Security & Performance Architecture

## Security Test Suite Architecture (`tests/security.rs`)

```rust
#[test]
fn path_traversal_payloads_rejected() {
    let tmp = TempDir::new().unwrap();
    // Test parent traversal (../), absolute path (/etc/passwd), and symlink traversal
}

#[test]
fn atomic_write_guarantees_integrity() {
    let tmp = TempDir::new().unwrap();
    // Test tempfile write + atomic rename without residual temp files
}
```

## Performance Benchmark Suite (`benches/benchmarks.rs`)

```rust
#[test]
fn benchmark_state_diff_and_workspace_overrides_under_50ms() {
    let start = std::time::Instant::now();
    // Perform state load with workspace overrides & diff computation
    let duration = start.elapsed();
    assert!(duration.as_millis() < 50, "State diff took {:?}", duration);
}
```
