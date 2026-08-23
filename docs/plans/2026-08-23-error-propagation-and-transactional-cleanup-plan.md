# Implementation Plan: Error Propagation & Transactional State Commit

- **Date:** 2026-08-23
- **Issue:** #162 (P1)
- **Origin:** `docs/brainstorms/2026-08-23-error-propagation-and-transactional-cleanup-requirements.md`
- **OpenSpec Change:** `error-propagation-transactional-cleanup`

---

## 🎯 Implementation Units

### U1: `uninstall.rs` Error Propagation & Transactional State
- **Files**: `src/commands/uninstall.rs`
- **Details**: Propagate `restore_latest`, `remove_file`, `remove_dir_all` errors via `?`. Log warnings for `SkillRegistry::remove`. Execute `state.save(&path)` only after required work succeeds.

### U2: `deinit_prj.rs` & `init_prj.rs` Error Propagation
- **Files**: `src/commands/deinit_prj.rs`, `src/commands/init_prj.rs`
- **Details**: Propagate `remove_file` and `write_atomic` errors via `?`. Log warnings for `SkillRegistry::sync_registry`. Execute `state.save(&path)` only after required work succeeds.

### U3: Verification & Failure Injection Tests
- **Files**: `tests/cli.rs`
- **Details**: Add tests for error propagation when target files or directories fail during `uninstall`, `deinit-prj`, or `init-prj`.
