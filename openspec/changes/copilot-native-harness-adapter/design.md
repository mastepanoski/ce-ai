# Design: GitHub Copilot Native Harness Adapter (Issue #177)

## 1. Structural Data Models & Schemas (`src/harness/copilot.rs`)

```rust
pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct CopilotAdapter;

impl HarnessAdapter for CopilotAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Copilot
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("mcp-config.json") {
            return home.to_path_buf();
        }

        if let Ok(config_env) = std::env::var("COPILOT_CONFIG_DIR") {
            return PathBuf::from(config_env).join("mcp-config.json");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".copilot") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".copilot").join("mcp-config.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CopilotMcpServer {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CopilotMcpConfig {
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: BTreeMap<String, CopilotMcpServer>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}
```

## 2. API Contract & Helper Functions

1. `pub fn register_copilot_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`:
   - Parses `config_path` if present using `CopilotMcpConfig`.
   - Inserts or updates `mcpServers.<name>` entry with `command`, `args`, and optional `env` map, preserving existing custom extra fields on the server.
   - Preserves all other top-level JSON keys.
   - Writes back atomically using `write_atomic`.

2. `pub fn unregister_copilot_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError>`:
   - Removes `name` from `mcpServers`.
   - Writes updated JSON document back atomically. Leaves `mcp-config.json` intact to preserve user options and OAuth credentials.

3. `pub fn update_copilot_instructions_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError>`:
   - Updates `.github/copilot-instructions.md` with demarcated `CE-AI MANAGED BLOCK`.

4. **Harness Probing & Detection**:
   - `HarnessKind::Copilot.is_installed_on_host`: Checks if `$COPILOT_CONFIG_DIR` exists, or if `~/.copilot` exists.
   - `HarnessKind::Copilot.is_ce_installed`: Checks if `mcp-config.json` contains `mcpServers` sidecars or if `skills/` exists under the harness directory.

5. **Backup & Health Check Integration**:
   - `backups.rs`: Recognized with `copilot-` prefix in backup storage.
   - `doctor.rs` & `status.rs`: Reads `mcp-config.json` `mcpServers` for `codegraph` / `engram` health status.
