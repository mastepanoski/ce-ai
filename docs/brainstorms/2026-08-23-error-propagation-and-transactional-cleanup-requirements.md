# Error Propagation & Transactional State Commit Requirements

- **Date:** 2026-08-23
- **Issue:** #162 (P1 — Destructive/cleanup errors suppressed in uninstall, deinit-prj and init-prj while reporting success)
- **Status:** Approved (Brainstorm Completed)
- **Scope Tier:** P1 Reliability & Integrity Fix

---

## 🎯 1. Overview & Problem Statement

Currently, `src/commands/uninstall.rs`, `src/commands/deinit_prj.rs`, and `src/commands/init_prj.rs` use `let _ =` to swallow errors during file deletion, backup restoration, atomic writes, and registry synchronization.

This leads to:
1. False success messages (`"✅ Uninstalled cleanly"`, `"✓ Removed project adoption block"`) printed even when filesystem mutations failed due to permission denied or IO errors.
2. State desynchronization: `state.json` is updated to claim completion even though the target files remain corrupted or un-deleted on disk.

This P1 defect is resolved by enforcing **strict error propagation** for required filesystem operations and **transactional state updates** (saving `state.json` ONLY after filesystem work succeeds).

---

## 🔒 2. Categorization of Operations & Policy

| Command | Operation | Classification | Error Policy |
| :--- | :--- | :--- | :--- |
| `uninstall` | Restore backup / delete target config | **Required** | Propagate error (`?`); suppress success msg; abort before `state.json` save. |
| `uninstall` | Remove `compound-engineering/` managed dir | **Required** | Propagate error (`?`); suppress success msg; abort before `state.json` save. |
| `uninstall` | Remove skill registry entry | **Best-Effort** | Emit `eprintln!("warning: ...")` on failure; proceed with state save. |
| `deinit-prj` | Remove `AGENTS.md` / `CLAUDE.md` | **Required** | Propagate error (`?`); suppress success msg; abort before `state.json` save. |
| `deinit-prj` | Atomic write of cleaned `AGENTS.md` | **Required** | Propagate error (`?`); suppress success msg; abort before `state.json` save. |
| `deinit-prj` | Clean `.gitignore` block / delete file | **Required** | Propagate error (`?`); suppress success msg; abort before `state.json` save. |
| `init-prj` | Inject `.gitignore` sentinel block | **Required** | Propagate error (`?`); suppress success msg; abort before `state.json` save. |
| `init-prj` | Sync skill registry | **Best-Effort** | Emit `eprintln!("warning: ...")` on failure; proceed with state save. |

---

## 📋 3. Operational Requirements

1. **Transactional State Commit**:
   - `state.save(&global_state_path)` MUST occur AFTER all required filesystem operations have completed successfully.
   - If any required filesystem operation fails, `state.json` MUST remain untouched, and the command MUST return a non-zero exit code (`CeError::IO` / `CeError::Runtime`).
2. **Zero False Success Messages**:
   - If any required step fails, no success message (`"✅ Uninstalled cleanly"`, `"✓ Adopted project"`) shall be printed.
3. **Explicit Warnings for Best-Effort Cleanups**:
   - Non-critical optional cleanups emit `eprintln!("warning: ...")` on failure instead of silently swallowing with `let _ =`.

---

## 🔄 4. OpenSpec Handoff & Next Steps

This requirements document is frozen in `docs/brainstorms/2026-08-23-error-propagation-and-transactional-cleanup-requirements.md`.

Next phase: **Stage 2 (OpenSpec Definition)** in `openspec/changes/error-propagation-transactional-cleanup/`:
- `proposal.md`
- `exploration.md`
- `design.md`
- `spec.md`
- `tasks.md`
