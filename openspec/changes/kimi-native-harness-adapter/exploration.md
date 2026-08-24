# Exploration: Kimi Code CLI Native Harness Adapter (Issue #178)

## Technical Architecture & Investigation

### 1. Kimi Code CLI Product & Layout (Official Moonshot AI Docs)
- **Executable**: `kimi` (Kimi Code CLI, `github.com/MoonshotAI/kimi-code`).
- **Config Directory**: `~/.kimi-code` (environment variable override: `$KIMI_CODE_HOME`). Legacy `~/.kimi` belongs to retired `kimi-cli`.
- **MCP Config File**: `~/.kimi-code/mcp.json`.
  - Schema:
    ```json
    {
      "mcpServers": {
        "server-name": {
          "command": "exec-name",
          "args": ["arg1", "arg2"],
          "env": { "KEY": "VAL" }
        }
      }
    }
    ```
- **Project Rules & Directives**: Global and project `AGENTS.md`.
- **Skills**: `$KIMI_CODE_HOME/skills/` (e.g. `~/.kimi-code/skills/`).

### 2. Migration Strategy from Legacy Generic Adapter
- Remove `HarnessKind::Kimi => home.join(".kimi").join("config.json")` from `src/harness/generic_json.rs`.
- Implement `KimiAdapter` in `src/harness/kimi.rs`.
- Wire `Kimi` across CLI subcommands (`install`, `tools`, `init-prj`, `deinit-prj`, `sync`, `uninstall`, `backups`).
