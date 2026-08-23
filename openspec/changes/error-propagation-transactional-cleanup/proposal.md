# OpenSpec Proposal: Error Propagation & Transactional Cleanup

- **Change:** `error-propagation-transactional-cleanup`
- **Issue:** #162 (P1)
- **Author:** Antigravity AI
- **Date:** 2026-08-23
- **Status:** Proposed

---

## 🎯 1. Problem Statement

Commands `uninstall`, `deinit-prj`, and `init-prj` contained several `let _ =` error-suppression lines during filesystem operations (`remove_file`, `remove_dir_all`, `write_atomic`). If an operation failed, `state.json` was updated to claim success anyway, and a positive status message was emitted to `stdout`.

---

## 🚀 2. Proposed Solution

1. Replace `let _ =` on required filesystem operations with proper error propagation (`?`) or explicit error logging (`eprintln!`).
2. Order mutations transactionally so `state.json` is updated and saved **only after** all required filesystem operations succeed.
3. Emit warnings on `stderr` for non-critical best-effort cleanups without failing the overall command.
