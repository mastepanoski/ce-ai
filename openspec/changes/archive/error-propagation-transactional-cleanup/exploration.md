# OpenSpec Exploration: Required vs Best-Effort Operation Ordering

- **Change:** `error-propagation-transactional-cleanup`
- **Issue:** #162 (P1)

---

## 🔍 1. Audit Findings

- `uninstall.rs`: `restore_latest`, `remove_file`, `remove_dir_all`, `SkillRegistry::remove` used `let _ =`.
- `deinit_prj.rs`: `remove_file` for `AGENTS.md`, `CLAUDE.md`, `.gitignore`, and `write_atomic` used `let _ =`.
- `init_prj.rs`: `.gitignore` `write_atomic` and `SkillRegistry::sync_registry` used `let _ =`.

---

## 💡 2. Architectural Decision

- Propagate all required file IO errors via `?`.
- Update state in memory during iteration, but execute `state.save(&path)` strictly as the final step after all filesystem operations succeed.
