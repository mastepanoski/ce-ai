---
title: "OpenCode V1/V2 Dual Plugin Loader, Version Detection & Native Commands"
domain: harnesses
version: 1.74.0
last_updated: "2026-09-30"
dependencies: [opencode, state, doctor]
---

# Specification: OpenCode V1/V2 Dual Plugin Loader, Version Detection & Native Commands

## WHEN `ce-ai doctor` checks OpenCode plugin health
- **THEN** it accepts the loader registration if either `"plugins"` (V2) or `"plugin"` (V1) in `opencode.json` contains the installed loader path.
- **AND** it verifies that the installed loader content matches the detected OpenCode environment (accepting V2 or dual shape for OpenCode 2.x, and V1 or dual shape for OpenCode 1.x).
- **AND** it reports "SessionStart plugin missing or outdated" if the loader file does not exist, fails version-specific shape validation, or is omitted from `opencode.json`.

## WHEN `ce-ai install --harness opencode` or `ce-ai sync` runs
- **THEN** it installs the embedded dual plugin loader to `<config_dir>/compound-engineering/plugins/compound-engineering.js`.
- **AND** if OpenCode V2 is active, it creates a symlink at `<config_dir>/plugins/compound-engineering.js` pointing to the managed loader so OpenCode V2's native plugin scanner activates it without path errors.
- **AND** if `opencode.json` already contains `"plugins"`, it appends to `"plugins"`; if it contains `"plugin"`, it appends to `"plugin"`; if neither exists, it prefers `"plugins"`.
- **AND** it materializes native command markdown files in `<config_dir>/commands/<skill_id>.md` for each user-invocable CE skill with frontmatter description and `<!-- ce-ai:managed-command -->` marker.
- **AND** re-running `sync` is completely idempotent, producing zero unnecessary mutations or file drift.

## WHEN OpenCode V2 initializes
- **THEN** it executes `setup(ctx)` from the default export with `id: "compound-engineering"`.
- **AND** it injects `ce-ai workflow resume` state on `session.created` via `ctx.session.synthetic`.
- **AND** it triggers turn-end checkpoint evaluation on `session.idle`.
- **AND** it appends repo state to `event.system` during `context` and `compaction` hooks.
- **AND** all `/ce-*` commands appear in `opencode api get /api/command`.

## WHEN OpenCode V1 initializes
- **THEN** it executes `server()` or imports the named `CompoundEngineeringPlugin`.
- **AND** it registers skills and commands via the `config` hook.
- **AND** it injects `ce-ai workflow resume` state on `session.created` via `client.session.prompt`.
- **AND** it appends repo state to context during compaction and system transform hooks.

## WHEN `ce-ai uninstall --harness opencode` runs
- **THEN** it removes the managed loader from both `"plugins"` and `"plugin"` in `opencode.json`.
- **AND** it removes `<config_dir>/plugins/compound-engineering.js` and `<config_dir>/compound-engineering/plugins/compound-engineering.js`.
- **AND** it scans `<config_dir>/commands/` and removes only files marked with `<!-- ce-ai:managed-command -->`, preserving all custom user commands.

## Acceptance Criteria
- `opencode api get /api/plugin` displays `compound-engineering` with status `active`.
- `opencode api get /api/command` displays all 34 `/ce-*` commands.
- `ce-ai doctor` reports 0 OpenCode plugin findings.
- OpenCode V1 unit tests and OpenCode V2 tests pass.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e` all pass.
