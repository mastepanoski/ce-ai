# Design: Codex Native Harness Adapter (Issue #175)

## 1. Structural Data Models & Schemas (`src/harness/codex.rs`)

```rust
#[derive(Debug, Default)]
pub struct CodexAdapter;

impl HarnessAdapter for CodexAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Codex
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".codex") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        if let Ok(config_env) = std::env::var("CODEX_CONFIG_DIR") {
            return PathBuf::from(config_env).join("config.toml");
        }

        home_dir.join(".codex").join("config.toml")
    }
}
```

### Native TOML `mcp_servers` Table Schema
`~/.codex/config.toml` uses TOML tables for MCP server definitions:
```toml
[mcp_servers.codegraph]
command = "codegraph"
args = ["mcp"]

[mcp_servers.engram]
command = "engram"
args = ["serve"]
```

## 2. API Contract & Helper Functions

1. `pub fn register_codex_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`:
   - Parses `config_path` if present using `toml::Table`.
   - Inserts or updates `[mcp_servers.<name>]` table with `command`, `args`, and optional `env` table.
   - Preserves all other top-level TOML keys and user `[mcp_servers]` tables.
   - Writes back atomically using `write_atomic`.

2. `pub fn unregister_codex_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError>`:
   - Removes `name` from `[mcp_servers]`.
   - Writes updated TOML document back atomically. Leaves `config.toml` intact to preserve user options and OAuth credentials.

3. `pub fn update_codex_agents_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError>`:
   - Priority path resolution:
     1. `./AGENTS.md` if present
     2. `.codex/AGENTS.md` if `.codex/` directory exists
     3. Default to `./AGENTS.md`
   - Updates demarcated `CE-AI MANAGED BLOCK`.

4. **Harness Probing & Detection**:
   - `HarnessKind::Codex.is_installed_on_host`: Checks if `CODEX_CONFIG_DIR` exists, or if `~/.codex` exists, or if `~/.codex/config.toml` exists.
   - `HarnessKind::Codex.is_ce_installed`: Checks if `~/.codex/config.toml` contains `[mcp_servers]` sidecars or if `~/.codex/skills/` exists.

5. **Subcommand & Health Check Integration**:
   - `doctor.rs` & `status.rs`: Native multi-harness probing reads `~/.codex/config.toml` `[mcp_servers]` for `codegraph` / `engram` health status.
