# Native Multi-Harness Directory Resolution Requirements

- **Date:** 2026-08-23
- **Issue:** #157 (P0 — Non-OpenCode harness install writes synthetic config under OpenCode dir; uninstall leaves artifacts)
- **Status:** Approved (Brainstorm Completed)
- **Scope Tier:** P0 Architecture & Harness Resolution Fix

---

## 🎯 1. Overview & Problem Statement

Currently, `src/commands/install.rs` passes `target_base_dir` (`~/.config/opencode/`) as the base directory for ALL harnesses during installation.

When installing a non-OpenCode harness (e.g. `ce-ai install --harness cursor`), the CLI creates synthetic config files (such as `~/.config/opencode/.cursorrules`) inside the **OpenCode** configuration directory. When uninstalling (`ce-ai uninstall --harness cursor`), it removes only the state entry and leaves those synthetic files behind in OpenCode's directory. `--harness all` sprays synthetic files across OpenCode's folder.

This P0 defect is resolved by establishing **native per-harness directory resolution** (`harness_dir(home_dir)`).

---

## 🚀 2. Native Harness Directory Mapping

Each harness resolves its own native configuration root relative to `home_dir`:

| Harness Identifier | Native Base Directory Path | Native Config File Name |
| :--- | :--- | :--- |
| `opencode` | `~/.config/opencode/` | `opencode.json` |
| `claude` | `~/.config/claude/` (and `~/.claude.json`) | `claude.json` |
| `cursor` | `~/.cursor/` | `mcp.json` |
| `pi` | `~/.pi/` | `config.json` |
| `copilot` | `~/.config/github-copilot/` | `config.json` |
| `codex` | `~/.config/codex/` | `config.json` |
| `grok` | `~/.config/grok/` | `config.json` |
| `kimi` | `~/.config/kimi/` | `config.json` |
| `agy` | `~/.gemini/antigravity-cli/` | `antigravity.json` |
| `deepseek` | `~/.config/deepseek/` | `config.json` |
| `fx` | `~/.config/fx/` | `config.json` |

---

## 🔒 3. Goals & Acceptance Criteria

1. **Native Directory Resolution**:
   - `HarnessKind::harness_dir(home_dir: &Path)` returns the native configuration root for the specific harness.
   - `install`, `sync`, and `uninstall` use `harness.harness_dir(home_dir)` rather than hardcoding `~/.config/opencode/`.
2. **Zero Synthetic Leakage into OpenCode**:
   - `ce-ai install --harness cursor` creates `~/.cursor/mcp.json` and `~/.cursor/compound-engineering/`. Zero files are created in `~/.config/opencode/`.
3. **Clean Uninstall Parity**:
   - `ce-ai uninstall --harness cursor` removes `~/.cursor/compound-engineering/` and cleans `~/.cursor/mcp.json`, leaving NO residual artifacts.
4. **`--harness all` Isolation**:
   - Installing `--harness all` provisions each active harness in its respective native directory (`~/.config/opencode/`, `~/.cursor/`, `~/.config/claude/`, etc.) without cross-directory contamination.

---

## 🔄 4. OpenSpec Handoff & Next Steps

This requirements document is frozen in `docs/brainstorms/2026-08-23-harness-containment-safety-gate-requirements.md`.

Next phase: **Stage 2 (OpenSpec Definition)** in `openspec/changes/harness-containment-safety-gate/`:
- `proposal.md`
- `exploration.md`
- `design.md`
- `spec.md`
- `tasks.md`
