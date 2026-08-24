> STATUS (v1.20.1): CeError exit-code contract live in src/error.rs. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Error Propagation & Transactional State Commit

- **Change:** `error-propagation-transactional-cleanup`
- **Issue:** #162 (P1)

---

## 📋 Task Checklist

- [ ] **Task 1**: Refactor `src/commands/uninstall.rs` to propagate file IO and backup restoration errors via `?`, log warnings for best-effort registry cleanup, and delay `state.save(&state_path)` until required work succeeds.
- [ ] **Task 2**: Refactor `src/commands/deinit_prj.rs` to propagate `remove_file` and `write_atomic` errors (including `.gitignore` cleanup) via `?` and delay `state.save(&global_state_path)` until required work succeeds.
- [ ] **Task 3**: Refactor `src/commands/init_prj.rs` to propagate `.gitignore` `write_atomic` errors via `?` before `state.save(&global_state_path)` and log warnings on `SkillRegistry::sync_registry` failure.
- [ ] **Task 4**: Add unit & integration tests for `uninstall`, `deinit-prj`, and `init-prj` verifying non-zero exit codes AND verifying `state.json` remains unmodified on failure.
- [ ] **Task 5**: Verify formatting (`cargo fmt --check`), clippy (`cargo clippy --all-targets --all-features -- -D warnings`), and test suite (`cargo test`).
