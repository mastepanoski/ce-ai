# Exploration: Grok Native Harness Adapter (Issue #176)

## 1. Official Grok Build Layout
- **Harness Directory**: `$GROK_HOME` if set, otherwise `$HOME/.grok`.
- **Config Path**: `<harness_dir>/config.toml`.
- **MCP Table Schema**:
  ```toml
  [mcp_servers.codegraph]
  command = "codegraph"
  args = ["mcp"]
  ```
- **Project Rule Location**: `.grok/rules/compound-engineering.md`.

## 2. Preservation Requirements
- Must preserve user TOML sections like `[cli]`, `[marketplace]`, and auth configuration.
- Must preserve unmanaged `[mcp_servers]` tables.
- Must never write OpenCode schema keys (`plugin`, `skills.paths`) into `config.toml`.

## 3. Uninstallation Policy
- Unregister `ce-ai` sidecars from `[mcp_servers]`.
- If `[mcp_servers]` table becomes empty, leave `config.toml` intact to preserve user CLI options and auth tokens.
- Clean managed skills directory `<harness_dir>/skills/`.
