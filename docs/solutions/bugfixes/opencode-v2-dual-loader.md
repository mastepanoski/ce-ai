---
title: "OpenCode V2 Dual Plugin Loader, Version-Aware Detection and Native Command Materialization"
category: "bugfixes"
date: "2026-09-30"
tags:
  - opencode
  - plugin-loader
  - v2-migration
  - esm
  - native-commands
  - doctor-false-positive
components:
  - opencode::plugins
  - opencode::config
  - commands::doctor
  - commands::sync
applies_when: "Running ce-ai with OpenCode 2.x, debugging missing slash commands (/ce-*), or resolving doctor SessionStart plugin warnings"
problem_type: "bugfix"
---

# OpenCode V2 Dual Plugin Loader, Version-Aware Detection and Native Command Materialization

## Problem
Users running OpenCode 2.x (`v2.0.20+`) experienced two major breakages when using `ce-ai`:
1. **Persistent False-Negative in `ce-ai doctor`**:
   The doctor command reported:
   `opencode: SessionStart plugin missing or outdated in '~/.config/opencode' — run 'ce-ai sync' or 'ce-ai install --harness opencode' to update`
   Running `ce-ai sync` or `ce-ai install` did not clear the warning.
2. **Disappearance of all `/ce-*` Slash Commands**:
   The 34 expected Compound Engineering slash commands (`/ce-brainstorm`, `/ce-plan`, `/ce-work`, `/lfg`, `/wtf`, `/sequential-thinking`) completely disappeared from the OpenCode prompt autocompletion.
   Meanwhile, agent skills remained callable directly by LLM models because `opencode.json` declared `skills.paths` statically.

## Root Cause Analysis
Two independent breaking changes in OpenCode 2.x caused this failure:

1. **Configuration Key Rename (`plugin` -> `plugins`)**:
   `src/opencode/plugins.rs` (`has_session_start_plugin`) and `src/opencode/config.rs` hardcoded lookups for `val.get("plugin")`. OpenCode V2 renamed this top-level configuration key in `opencode.json` to `"plugins"`. Because the detector only inspected `"plugin"`, it never saw the registered loader under `"plugins"`, causing an unrecoverable false-negative.
2. **Stricter ESM Export Contract & Runtime Command Hooks**:
   The embedded loader script (`.opencode/plugins/compound-engineering.js`) previously used the V1 named export format:
   `export const CompoundEngineeringPlugin = async ({ ... }) => ({ config, event, ... })`
   OpenCode 2.x rejected this module with:
   `PluginModule.LoadError: Plugin must export a default definition with an id and an effect or setup function. (cause: SchemaError(Missing key at ["default"]))`
   Because V1 registered slash commands dynamically at runtime via the `config.command[name]` hook inside the plugin, the failure of the plugin to load prevented all slash commands from being registered.

## Solution Architecture

To deliver robust cross-version compatibility without regressing OpenCode 1.x users, the solution implements a three-pronged architecture:

### 1. Dual-Compatible ESM Plugin Loader (`.opencode/plugins/compound-engineering.js`)
The loader exports a hybrid interface that satisfies both V1 and V2 runtime contracts simultaneously:
- **V2 Export**: `default.setup(ctx)` with `id: "compound-engineering"`. Subscribes to lifecycle events via `ctx.event.subscribe` and hooks into compaction and context transforms.
- **V1 Export**: Named `CompoundEngineeringPlugin` and `default.server()`.
- **Workflow Resume Preservation**: Both paths execute `spawnSync("ce-ai", ["workflow", "resume", ...])` during Turn-0 lifecycle events (`session.created`, `session.idle`, compaction, and context transform).
- **Dynamic Skills Directory Resolution**: Traverses candidate sibling directories (`../../skills`, `../compound-engineering/skills`, `../skills`) to locate active skills regardless of directory depth.

### 2. Version-Aware Detection & Config Key Agnosticism (`src/opencode/plugins.rs` & `config.rs`)
- **Version Probe**: `OpenCodeVersion::detect()` runs `opencode --version` to identify `V1`, `V2`, or `Unknown`.
- **Key Agnosticism**: Functions reading or mutating `opencode.json` inspect both `"plugins"` and `"plugin"` arrays. When registering new plugins, V2 uses `"plugins"` while V1 uses `"plugin"`.
- **Shape Validation**: `is_valid_loader_content()` verifies the loader matches the expected API shape of the target OpenCode version rather than relying on a naive substring check.
- **Self-Repair on Sync**: `ensure_session_start_plugin()` detects stale on-disk loaders and overwrites them with the updated dual loader.

### 3. Native Command Materialization (`src/opencode/plugins.rs`)
In OpenCode V2, runtime dynamic command registration via `config` hook is deprecated in favor of declarative files. `ce-ai` now materializes each user-invocable skill into `~/.config/opencode/commands/<command-name>.md`:
- Includes YAML frontmatter with `description` and prompt templates utilizing `$ARGUMENTS`.
- Employs a tamper-evident HTML comment guard: `<!-- ce-ai:managed-command -->`.
- Preserves all unmanaged custom user commands (`sdd-*.md`, `skill-*.md`) during sync and cleanly removes only managed commands during uninstallation.
- Places the plugin file into `~/.config/opencode/plugins/compound-engineering.js` where OpenCode V2 autodiscovery registers it cleanly.

## Verification & Empirical Evidence
- **Live OpenCode Daemon API (`opencode api get /api/plugin`)**:
  Verified `compound-engineering` plugin loaded in status `"active"` with `"features": { "server": true }` on macOS with OpenCode `v2.0.20`.
- **Command Palette API (`opencode api get /api/command`)**:
  Confirmed all 37 commands (`/ce-brainstorm`, `/ce-plan`, `/ce-work`, `/lfg`, `/wtf`, `/sequential-thinking`) indexed and executable.
- **`ce-ai doctor`**: Verified 0 warnings reported for OpenCode.
- **Automated Tests**: Added 8 unit tests in `src/opencode/tests/plugins.rs` covering dual-key detection, version parsing, and managed command lifecycle. Passed `cargo test` (267+ tests) and Docker E2E (`make e2e`).

## Key Learnings
1. **Never Assume Plugin Registration Hooks Are Stable Across Majors**: CLI hosts evolve from runtime JS hooks (`config.command`) to declarative file autodiscovery (`commands/*.md`). Building native file materialization provides higher reliability and lower execution overhead.
2. **Dual-Export Pattern Prevents Ecosystem Fragmentation**: Exposing both `default.setup(ctx)` and `server()` in a single JavaScript file avoids maintaining multiple divergent loader assets and eliminates configuration branching.
