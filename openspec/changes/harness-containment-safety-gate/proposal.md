# OpenSpec Proposal: Native Per-Harness Directory Resolution

- **Change:** `harness-containment-safety-gate`
- **Issue:** #157 (P0)
- **Author:** Antigravity AI
- **Date:** 2026-08-23
- **Status:** Proposed

---

## 🎯 1. Problem Statement

`src/commands/install.rs` previously hardcoded `target_base_dir` (`~/.config/opencode/`) for all harnesses during installation and uninstallation.

Installing a non-OpenCode harness (e.g. `ce-ai install --harness cursor`) created synthetic `.cursorrules` inside `~/.config/opencode/`. Uninstallation removed state entries while leaving those synthetic files behind in OpenCode's directory.

---

## 🚀 2. Proposed Solution

Establish `HarnessKind::harness_dir(home_dir)` as the single source of truth for native harness directories:
1. `opencode`: `~/.config/opencode/`
2. `claude`: `~/.config/claude/`
3. `cursor`: `~/.cursor/`
4. `pi`: `~/.pi/`
5. `copilot`: `~/.config/github-copilot/`
6. `agy`: `~/.gemini/antigravity-cli/`
7. `codex`: `~/.config/codex/`
8. `grok`: `~/.config/grok/`
9. `kimi`: `~/.config/kimi/`
10. `deepseek`: `~/.config/deepseek/`
11. `fx`: `~/.config/fx/`

All lifecycle operations (`install`, `sync`, `uninstall`, `models set`) resolve the harness's native directory via `harness_dir`, ensuring zero cross-directory contamination.
