---
title: "Kimi Code Native Plugin Divergence: Visibility, Not Control (Issue #349 POV)"
category: "architecture"
date: "2026-09-10"
tags:
  - kimi
  - doctor
  - plugins
  - divergence
  - architecture
  - pov
components:
  - harness::kimi
  - commands::doctor
applies_when: "Deciding whether ce-ai should mutate or trigger native plugin updates for Kimi Code, or evaluating future harness plugin manager boundaries"
---

# Kimi Code Native Plugin Divergence: Visibility, Not Control

## Problem & Context
`ce-ai` manages isolated installations of `compound-engineering` across configured harnesses (e.g. `~/.kimi-code/compound-engineering/`, `~/.claude/compound-engineering/`), maintaining state in `state.json`.

In Kimi Code, an internal native plugin management subsystem exists under `~/.kimi-code/plugins/` (tracked in `installed.json` and mirrored in `~/.kimi-code/plugins/managed/<plugin-id>/`). In v1.48.0, `ce-ai doctor` introduced read-only divergence detection (`src/harness/kimi.rs:193`, `src/commands/doctor.rs:289-300`) to alert users when Kimi's native plugin version lags behind the `ce-ai` managed tree:

```
doctor-info: kimi native plugin divergence detected for 'compound-engineering' : native plugin manager has v3.14.3 but ce-ai managed harness is v3.24.0 (update the plugin via Kimi's native plugin manager, or uninstall the native plugin to use the ce-ai managed tree exclusively)
```

[Issue #349](https://github.com/mastepanoski/ce-ai/issues/349) posed the question: **should `ce-ai` trigger or manage the update of Kimi Code's native plugin automatically, or does the "visibility, not control" principle established in [Issue #327](https://github.com/mastepanoski/ce-ai/issues/327) for Claude Code apply equally here?**

## Architectural Decision: Reject (Visibility, Not Control)

Following the formal `ce-pov` evaluation methodology (Tier 2 bounded capability decision), the verdict is **Reject**: `ce-ai` will **not** attempt automated updates of Kimi Code's native plugins, preserving the exact same architectural boundary established for Claude Code in [Issue #327](https://github.com/mastepanoski/ce-ai/issues/327) and documented in [`docs/solutions/architecture/claude-native-marketplace-divergence.md`](claude-native-marketplace-divergence.md).

`ce-ai` remains strictly a companion of observability:
1. `src/harness/kimi.rs:193` (`check_kimi_marketplace_divergence`) continues as a pure read-only check.
2. `src/commands/doctor.rs:289-300` continues reporting advisory non-blocking `doctor-info` / `doctor-warn` diagnostics.
3. Remediation remains manual: the user updates the plugin interactively within Kimi Code (`/plugins`) or uninstalls the native plugin to rely exclusively on the `ce-ai` managed harness.

## Empirical Evidence & Host Investigation

Rather than relying on analogy with Claude Code, this decision is backed by empirical inspection of the host system and reverse engineering of the Kimi Code binary (`/Users/mastepanoski/.kimi-code/bin/kimi`, v0.42.0, Mach-O 64-bit arm64):

### 1. Inexistence of a Non-Interactive Plugin CLI
- `kimi --help` lists subcommands for `export`, `fork`, `provider`, `session`, `acp`, `web`, `server`, `rc`, `login`, `doctor`, `vis`, `migrate`, and `upgrade`.
- There is **no `plugin` subcommand**; invoking `kimi plugin --help` falls through to the top-level help. Kimi exposes zero non-interactive CLI commands for plugin installation or updates.

### 2. Runtime Encapsulation Behind Interactive TUI
- Analysis of binary symbols reveals that plugin management logic is encapsulated in an internal `PluginManager` class with RPC methods (`listPlugins`, `installPlugin`, `setPluginEnabled`, `removePlugin`, `reloadPlugins`).
- These methods are exposed exclusively through the interactive TUI modal `/plugins` (`plugins-selector.ts`, dialog tabs `installed`, `official`, `third-party`, `custom`).
- Triggering an update requires an active interactive session where the user selects the update badge (`installedHint: ... Enter update ...`). There is no scriptable endpoint or flag.

### 3. Filesystem Layout in `~/.kimi-code/plugins/`
- **Metadata Index**: `~/.kimi-code/plugins/installed.json` maintains an array of records (`id`, `root`, `source`, `enabled`, timestamps, `originalSource`, and GitHub ref info).
- **Installed Tree**: Plugins reside as full copied directories under `~/.kimi-code/plugins/managed/<plugin-id>/` with their own `package.json` and `kimi.plugin.json` (or `.kimi-plugin/plugin.json`). They are not symlinks.

### 4. Severe Concurrency and State Clobbering Hazards
Writing directly to `~/.kimi-code/plugins/managed/` or mutating `installed.json` from `ce-ai` introduces high operational risk:
- **No Concurrency Locks**: Unlike Claude Code (which creates `.in_use/<pid>` locks to guard active sessions), Kimi Code implements **no filesystem locks** around `plugins/` or `installed.json`.
- **In-Memory Cache Desynchronization**: Kimi Code **does not watch** `installed.json` via `fs.watch`. At startup, `PluginManager` loads `installed.json` into an in-memory `Map` (`this.records`). Any external write by `ce-ai` while Kimi is running is invisible to Kimi until `/plugins reload` or process restart.
- **Silent Overwrites (State Clobbering)**: Whenever Kimi performs any internal mutation (e.g. user toggles a plugin, enables an MCP server, or installs another plugin), `PluginManager.persist()` serializes its in-memory snapshot and atomically renames `.installed.json.tmp` over `installed.json`. **This silently obliterates any external changes written by `ce-ai`.**

## Generalization: The "Visibility, Not Control" Pattern

This is the second time the question of taking over an external tool's native plugin manager has been investigated and rejected:
1. **Claude Code ([Issue #327](https://github.com/mastepanoski/ce-ai/issues/327))**: Claude caches native plugins under `~/.claude/plugins/cache/` and tracks them in `installed_plugins.json`. `ce-ai` rejected direct mutation and settled on `doctor-info` visibility.
2. **Kimi Code ([Issue #349](https://github.com/mastepanoski/ce-ai/issues/349))**: Kimi maintains copies under `~/.kimi-code/plugins/managed/` and tracks them in `installed.json`. Direct mutation was rejected due to lack of a scriptable CLI and state clobbering risks.

### Durable Rule for Future Harnesses
When integrating new AI coding harnesses (#7, #8, etc.) that maintain their own native plugin or extension ecosystems:
> **If the harness maintains an internal plugin tree without an official, stable, non-interactive CLI for lifecycle management, `ce-ai` MUST adopt "visibility, not control": detect and report version drift via `ce-ai doctor`, guide the user to the tool's native update UI, and NEVER mutate the external tool's private registry or directories directly.**

## Reversal Trigger (Tier 2)

This decision is governed by a clear reversal trigger:
- If a future release of Kimi Code introduces an official, non-interactive CLI command (e.g. `kimi plugin update <id>` or `kimi plugin install --force <id>`) with deterministic exit codes and non-interactive output, `ce-ai` may re-open this decision to evaluate invoking that official CLI.
- Direct filesystem mutation of `~/.kimi-code/plugins/` will remain permanently rejected.

## Related Documentation & Issues
- [Issue #349: Question on Kimi native plugin update automation](https://github.com/mastepanoski/ce-ai/issues/349)
- [Issue #327: Claude native plugin divergence detection](https://github.com/mastepanoski/ce-ai/issues/327)
- [Sibling Decision: Claude Code Native Plugin Divergence](claude-native-marketplace-divergence.md)
- [`src/harness/kimi.rs`](../../src/harness/kimi.rs) — `check_kimi_marketplace_divergence`
- [`src/commands/doctor.rs`](../../src/commands/doctor.rs) — `doctor-info` reporting
