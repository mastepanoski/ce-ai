# Implementation Plan: Per-Harness Native Directory Resolution

- **Date:** 2026-08-23
- **Issue:** #157 (P0)
- **Origin:** `docs/brainstorms/2026-08-23-harness-containment-safety-gate-requirements.md`
- **OpenSpec Change:** `harness-containment-safety-gate`

---

## 🎯 Implementation Units

### U1: `HarnessKind::harness_dir(home_dir)` Core Helper
- **Files**: `src/harness/mod.rs`
- **Details**: Maps all 12 `HarnessKind` variants (`opencode`, `claude`, `cursor`, `pi`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`) to their native host directory structures.

### U2: Lifecycle Command Refactoring (`install`, `uninstall`, `sync`, `models set`)
- **Files**: `src/commands/install.rs`, `src/commands/uninstall.rs`, `src/commands/sync.rs`, `src/commands/models.rs`
- **Details**: Resolves `harness_dir(&home_dir)` per harness during lifecycle operations, ensuring zero synthetic file leakage into `~/.config/opencode/`.

### U3: Unit & CLI Integration Verification
- **Files**: `src/harness/mod.rs`, `tests/cli.rs`
- **Details**: Tests `harness_dir` mappings unit-level and verifies CLI `install --harness cursor` and `uninstall --harness cursor` behavior in `tests/cli.rs`.
