# OpenSpec Exploration: Harness Directory Resolution Options

- **Change:** `harness-containment-safety-gate`
- **Issue:** #157 (P0)

---

## 🔍 1. Investigation

- **Previous behavior**: `install.rs` used `target_base_dir` (`~/.config/opencode`) for all harnesses.
- **New behavior**: `HarnessKind::harness_dir(home_dir)` maps each harness to its native directory.

---

## 💡 2. Architectural Decision

Implement `harness_dir(home_dir)` on `HarnessKind` and update `install.rs`, `uninstall.rs`, and `sync.rs` to compute `let harness_base_dir = harness_kind.harness_dir(&home);`.
