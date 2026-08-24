# Exploration: Google Antigravity (agy) Native Harness Adapter

## Technical Investigation

### Antigravity Directory Layout
Based on official Google Antigravity documentation:
- Root directory: `~/.gemini` (overridden by `$ANTIGRAVITY_CONFIG_DIR` or `$GEMINI_HOME`).
- Global MCP Config: `~/.gemini/config/mcp_config.json`.
- Global Skills Root: `~/.gemini/config/skills/` (primary) and `~/.gemini/antigravity-cli/skills/` (CLI-only).
- Workspace Rules: `.agents/rules/compound-engineering.md` and project root `GEMINI.md`.

### MCP Server JSON Schema (`~/.gemini/config/mcp_config.json`)
```json
{
  "mcpServers": {
    "codegraph": {
      "command": "codegraph",
      "args": ["mcp"],
      "env": {}
    },
    "remote_example": {
      "serverUrl": "https://mcp.example.com/sse",
      "headers": {
        "Authorization": "Bearer token"
      }
    }
  }
}
```

Key schema rules:
1. Top-level key MUST be `"mcpServers"`.
2. Stdio servers use `"command"` (string), `"args"` (array of strings), and `"env"` (map of string to string).
3. Remote servers MUST use `"serverUrl"` (string), NOT `"url"` or `"httpUrl"`.
4. Extra properties in user entries must be preserved cleanly across edits.
