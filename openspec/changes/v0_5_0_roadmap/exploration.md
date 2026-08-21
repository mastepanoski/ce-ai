# OpenSpec Exploration: Technical Tradeoffs for Release v0.5.0

## Scope Selection (Issue #7)
- **Workspace Scope**: Resolves `git rev-parse --show-toplevel` and writes `./.opencode/` or `./.claude/` locally within the repository.
- **Global Scope (Default)**: Writes to `~/.config/opencode/` or `~/.claude.json`.

## Companion Tools Manager (Issue #9)
- Probe system `PATH` using `which` / `where` and inspect harness MCP configs (`mcp_servers` in `opencode.json`).
- Register MCP servers for Engram, CodeGraph, Context7, and RTK in `opencode.json` and `.cursorrules`.

## Workflow FSM & Recovery (Issue #10)
- Track 7-stage development cycle in `state.json` under `workflow_state`.
- Parse `openspec/changes/*/tasks.md` to compute progress percentages and display stage status.
