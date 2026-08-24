# Task Breakdown: Real Long-Running `sync --watch` Loop & Drift Recovery

- [x] Add `interval_ms` and `max_passes` flags to `sync::Args` in `src/commands/sync.rs`
- [x] Implement `run_watch` polling loop and `check_and_repair_drift` in `src/commands/sync.rs`
- [x] Add integration test in `tests/cli.rs` asserting drift restoration mid-watch
- [x] Pass `cargo fmt`, `cargo clippy`, and `cargo test`
