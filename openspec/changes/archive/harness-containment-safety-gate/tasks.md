# OpenSpec Tasks: Per-Harness Native Directory Resolution

- **Change:** `harness-containment-safety-gate`
- **Issue:** #157 (P0)

---

## 📋 Task Checklist

- [x] **Task 1**: Implement `HarnessKind::harness_dir(home_dir)` in `src/harness/mod.rs`.
- [x] **Task 2**: Refactor `src/commands/install.rs` to compute `let harness_dir = harness_kind.harness_dir(&home_dir);` per harness.
- [x] **Task 3**: Refactor `src/commands/uninstall.rs` to compute `let harness_dir = harness_kind.harness_dir(&home_dir);` per harness during uninstall.
- [x] **Task 4**: Refactor `src/commands/sync.rs` and `src/commands/models.rs` to compute `let harness_dir = harness_kind.harness_dir(&home_dir);`.
- [x] **Task 5**: Add unit test for `harness_dir` in `src/harness/mod.rs` and CLI integration tests in `tests/cli.rs` for `install` and `uninstall` targeting native directories (e.g. `~/.cursor/`).
- [x] **Task 6**: Verify formatting (`cargo fmt --check`), clippy (`cargo clippy --all-targets --all-features -- -D warnings`), and test suite (`cargo test`).
