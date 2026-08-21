---
title: "Workspace Configuration Overrides (.ce-ai.json) & Complete Multi-Harness Uninstall Parity"
category: "architecture"
date: "2026-08-21"
tags:
  - workspace-overrides
  - state-precedence
  - multi-harness-uninstall
  - lifecycle-management
components:
  - state
  - uninstall
  - harness
applies_when: "Adding repository-local configuration overrides (.ce-ai.json) or extending multi-harness uninstall parity in ce-ai"
---

# Workspace Configuration Overrides (.ce-ai.json) & Complete Multi-Harness Uninstall Parity

## Context

Prior to Release v0.7.0, `ce-ai` stored model assignments (`ce-work`, `ce-plan`) strictly in the global user configuration (`~/.config/ce-ai/state.json`). Teams working on different repositories could not specify project-level LLM overrides without mutating global developer state. Additionally, `ce-ai uninstall` was OpenCode-specific and did not offer a complete removal path for managed loaders and skills across all installed harnesses (`--harness all --all`).

---

## Guidance & Architecture Patterns

### 1. Key-Level Workspace Merging Engine (`src/state/state.rs`)
- **Pattern**: Implement `State::load_with_workspace_overrides(global_path, workspace_root)`:
  - Reads baseline `~/.config/ce-ai/state.json`.
  - Checks if `.ce-ai.json` exists in `workspace_root`.
  - Merges field-by-field: local `model_assignments` override matching global slots while un-specified slots inherit global defaults.
- **Mental Model**: *Local Cockpit Presets vs Master Flight Plan*: Repositories adjust local model slots for specific tasks without altering standard developer defaults.

### 2. Multi-Harness Uninstall Parity (`src/commands/uninstall.rs`)
- **Pattern**: Extend `ce-ai uninstall` with `--harness <name|all>`, `--all`, and `--yes` / `-y` flags:
  - Restores pre-install backup configs for targeted harnesses.
  - Deletes managed loader scripts (`compound-engineering.js`), skill directories (`.opencode/skills/`), and SHA256 manifests when `--all` is passed.
  - Enforces interactive confirmation prompt unless `--yes` / `-y` is supplied for non-interactive scripts.
  - Retains all unmanaged user plugins and custom skills.

---

## Why This Matters

1. **Repo Isolation**: Teams can check in `.ce-ai.json` to enforce standard model roles (e.g., assigning a specialized local model to `ce-work`) per repository.
2. **Clean Life-Cycle Cleanup**: Users can completely purge managed plugins and loaders across all installed harnesses with a single command: `ce-ai uninstall --harness all --all --yes`.

---

## Code Examples

### Loading State with Overrides (`src/state/state.rs`):
```rust
let loaded = State::load_with_workspace_overrides(&global_path, Some(&ws_dir))?;
// loaded.model_assignments contains merged keys with local precedence
```

### Complete Uninstall CLI Command (`src/commands/uninstall.rs`):
```bash
ce-ai uninstall --harness all --all --yes
```
