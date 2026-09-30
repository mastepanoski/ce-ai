# Proposal: OpenCode V1/V2 Dual Plugin Loader, Version-Aware Detection & Native Commands

## Problem Statement
Users with OpenCode V2 report persistent false-negative health findings during `ce-ai doctor`:
```
opencode: SessionStart plugin missing or outdated in '~/.config/opencode' — run 'ce-ai sync' or 'ce-ai install --harness opencode' to update
```
Additionally, all `/ce-*` slash commands (34 compound engineering skills plus `lfg`, `wtf`, and `sequential-thinking`) fail to appear in OpenCode V2 UI and CLI, despite the skill paths being properly declared in `opencode.json`.

Root cause analysis isolated two defects:
1. **Hardcoded V1 Configuration Key Detection**:
   `has_session_start_plugin()` in `src/opencode/plugins.rs` explicitly queries `val.get("plugin").and_then(|p| p.as_array())`. OpenCode V2 renamed the configuration key from `plugin` to `plugins`. The detector therefore reports "missing or outdated" even when the loader is correctly registered under `plugins`. Furthermore, `merge_plugin` in `src/opencode/config.rs` only appends to `plugin`, creating invalid or ignored configurations on V2.
2. **Incompatible Plugin Loader Export & Command Registration in V2**:
   OpenCode V1 expected a function export or `{ server: async (...) => hooks }`. OpenCode V2 requires an object default export with `id` and `setup(ctx)` (or `Plugin.define({ id, setup })`). An outdated or V1 loader causes OpenCode V2 to fail with:
   `PluginModule.LoadError: Plugin must export a default definition with an id and an effect or setup function. (cause: SchemaError(Missing key at ["default"]))`.
   Consequently, the runtime `config` hook that registered skill slash commands never runs, breaking `/ce-*` discovery in OpenCode V2. Moreover, OpenCode V2 disallows file paths in `plugins: [...]` in `opencode.json` (logging `configured plugin path must be a directory`) and instead natively discovers `{command,commands}/**/*.md`.

## In-Scope Boundaries
- **Dual Compatibility Loader**: Rewrite `.opencode/plugins/compound-engineering.js` to expose a dual interface: V1 `server` / named export `CompoundEngineeringPlugin` and V2 `setup(ctx)` on default export with `id: "compound-engineering"`.
- **Version-Aware & Key-Agnostic Plugin Detection**: Update `has_session_start_plugin`, `ensure_session_start_plugin`, `remove_session_start_plugin`, and `src/opencode/config.rs` (`merge_plugin`) to accept and preserve both `plugins` (V2) and `plugin` (V1). Validate the actual loader form based on detected or supported OpenCode runtime versions.
- **Self-Repairing Loader Convergence**: Fix `ensure_session_start_plugin` to evaluate `is_valid_loader_content` rather than just `!s.contains("session.created")`, ensuring stale V1 loaders on disk are properly upgraded during `sync` and `install`.
- **Materialized Native Commands for OpenCode V2**: In addition to the runtime transform fallback, materialize managed slash commands under `<config_dir>/commands/<name>.md` with `<!-- ce-ai:managed-command -->` markers for all user-invocable skills, ensuring instantaneous native command availability in OpenCode V2.
- **Zero V1 Regressions**: Ensure OpenCode 1.x installations retain their functional commands via the `config` hook, state delivery via `session.created` prompt injection, and green `doctor` checks.

## Out-of-Scope Boundaries
- Modifying other harness integrations (Claude Code, OpenAI Codex, GitHub Copilot, Cursor).
- Changing OpenCode MCP server registrations (`codegraph`, `engram`, `context7`).
- Altering user-defined custom plugins or non-managed commands in `commands/`.

## Risk Evaluation & Mitigations
- **Risk**: OpenCode V2 fails if a file path is added to `plugins` array in `opencode.json`.
  - *Mitigation*: Ensure `plugins` in `opencode.json` only holds directories or package names when V2 is active, and place the plugin in `<config_dir>/plugins/compound-engineering.js` (or symlink from `<config_dir>/compound-engineering/plugins/`) where V2 auto-scans it, while keeping `<config_dir>/compound-engineering/plugins/` registered in `"plugin"` for V1.
- **Risk**: Clobbering user custom commands in `~/.config/opencode/commands/`.
  - *Mitigation*: Pre-check existing command files; only write managed files that carry the explicit `<!-- ce-ai:managed-command -->` header or match CE skill IDs, and preserve any user-owned markdown command.
- **Risk**: Dual export incompatibility across node/bun runtimes.
  - *Mitigation*: Vanilla ES module object export without requiring synthetic external `@opencode/plugin` imports, verified with live OpenCode V2 runtime and V1 unit test fixtures.

## Success Criteria
1. `ce-ai doctor` returns clean 0 for OpenCode with no false-negative "missing or outdated" warnings under both V1 and V2 configs.
2. All 34 `/ce-*` commands + `lfg`, `wtf`, `sequential-thinking` appear in `opencode api get /api/command`.
3. OpenCode V2 reports `compound-engineering` plugin as `active` in `opencode api get /api/plugin` with zero load errors.
4. `cargo fmt`, `cargo clippy`, `cargo test`, and `make e2e` pass with 100% green status.
