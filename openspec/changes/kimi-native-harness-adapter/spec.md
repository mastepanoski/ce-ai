# Specification: Kimi Code CLI Native Harness Adapter (Issue #178)

## Acceptance Criteria

### R1: Native Directory & Environment Override Resolution
- WHEN resolving `harness_dir(Kimi)` THEN `ce-ai` MUST check `$KIMI_CODE_HOME` first, falling back to `~/.kimi-code`.
- Legacy `~/.kimi/config.json` path is bypassed by native Kimi adapter operations.

### R2: Native `mcpServers` JSON Schema
- WHEN registering MCP servers for Kimi THEN `ce-ai` MUST write entries to `mcp.json` under top-level key `mcpServers` in format `{"command": "...", "args": [...], "env": {...}}`.
- `ce-ai` MUST NOT insert OpenCode specific keys (`plugin`, `skills.paths`).
- `ce-ai` MUST preserve unmanaged user servers and top-level JSON fields.

### R3: Project Rule Adoption
- WHEN running `init-prj` for Kimi THEN `ce-ai` MUST adopt project `AGENTS.md` with `CE-AI MANAGED BLOCK`.

### R4: Skills Installation & Clean Lifecycle
- WHEN installing skills for Kimi THEN `ce-ai` MUST place skill folders into `$KIMI_CODE_HOME/skills/` (or `~/.kimi-code/skills/`).
- WHEN running `ce-ai uninstall --harness kimi` THEN `ce-ai` MUST unregister managed sidecars and clean managed skills from `~/.kimi-code/skills/` while preserving user custom servers and skills without touching legacy `~/.kimi/`.
