# Proposal: Pi Native Harness Adapter

## Problem Statement
`ce-ai`'s previous stub handling for `pi` assumed a fictional `~/.pi/config.json` file with OpenCode-style schema.
According to official `pi` coding agent documentation (pi.dev / `@earendil-works/pi-coding-agent`):
- Managed assets live under `~/.pi/agent/` (overridden by `$PI_CODING_AGENT_DIR`).
- `pi` intentionally has **no native MCP server configuration** ("No MCP philosophy").
- `pi` loads instructions from `AGENTS.md` and `.pi/AGENTS.md`.
- `pi` loads skills from `~/.pi/agent/skills/`.

## In-Scope
- Implement `PiAdapter` in `src/harness/pi.rs` with `~/.pi/agent/` home directory (honoring `$PI_CODING_AGENT_DIR`).
- Install managed skills to `~/.pi/agent/skills/` without fabricating JSON plugin or MCP configuration files.
- Report MCP as unsupported when `ce-ai tools install` targets `pi`.
- Adopt `.pi/AGENTS.md` when `.pi/` directory exists in project root.
- Clean uninstall lifecycle removing `~/.pi/agent/skills/`.

## Out-of-Scope
- Writing MCP JSON configs for `pi` (unsupported by upstream `pi`).
