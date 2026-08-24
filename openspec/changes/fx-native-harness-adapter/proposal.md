# Proposal: fx Native Harness Adapter

## Problem Statement
`ce-ai` previously mapped `fx` to a fictional `~/.config/fx/fx.json` file with OpenCode-style schema.
According to official `fx` coding agent documentation (fx.sh / `github.com/vercel-labs/fx`):
- Managed assets live under `~/.fx/` (environment variable override `$FX_HOME`).
- MCP configuration lives in `~/.fx/mcp.json` using root key `mcp` (`{"mcp": {"<name>": {"type": "local", "command": ["<cmd>", "<args>..."], "environment": {}}}}`).
- Skills live under `~/.fx/skills/`.

## In-Scope
- Implement native `FxAdapter` in `src/harness/fx.rs`.
- Target `~/.fx/mcp.json` with `mcp` root key and array-form `command` for MCP servers.
- Target `~/.fx/skills/` for managed skills installation.
- Adopt project rules in `AGENTS.md` and `.fx/AGENTS.md` (when `.fx/` directory pre-exists) during `ce-ai init-prj` and `deinit-prj`.
- Support `$FX_HOME` environment variable override for custom directory relocation.
- Wire `HarnessKind::Fx` across all commands and remove `fx` from `src/harness/generic_json.rs`.
- Full lifecycle uninstall cleaning up `~/.fx/mcp.json` entries and `~/.fx/skills/` while preserving user files.

## Out-of-Scope
- Supporting fictional `~/.config/fx/fx.json` paths.
