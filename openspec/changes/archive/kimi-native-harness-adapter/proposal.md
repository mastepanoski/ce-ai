# Proposal: Kimi Code CLI Native Harness Adapter (Issue #178)

- **Goal**: Upgrade Kimi Code CLI (`kimi`) harness adapter from generic JSON (`~/.kimi/config.json`) to native format (`~/.kimi-code/mcp.json` and `$KIMI_CODE_HOME/skills/`).

## Problem Statement
Currently, `ce-ai` treats Kimi as a generic JSON harness at `~/.kimi/config.json`. Official Moonshot AI documentation (Kimi Code CLI, `kimi`) specifies `~/.kimi-code/` as the native configuration root (overridden by `$KIMI_CODE_HOME`), `~/.kimi-code/mcp.json` for MCP server registration (`mcpServers` JSON object), `~/.kimi-code/skills/` for agent skills, and `AGENTS.md` for instructions.

## Scope & Success Criteria
1. `HarnessKind::Kimi` resolves harness directory to `~/.kimi-code` (honoring `$KIMI_CODE_HOME`).
2. MCP servers register natively in `~/.kimi-code/mcp.json` under `mcpServers.<name>` (`{"command": "...", "args": [...], "env": {...}}`), preserving user entries and top-level JSON keys.
3. Zero OpenCode keys (`plugin`, `skills.paths`) written to `mcp.json`.
4. Skills installed to `~/.kimi-code/skills/`.
5. Clean uninstallation: unregister sidecars, remove managed skills, preserve user custom servers and skills.
6. Project rule adoption targets project `AGENTS.md` with `CE-AI MANAGED BLOCK`.
