# Technical Design: Cursor Native Harness Adapter

## 1. Data Schemas

### `CursorMcpConfig` (`~/.cursor/mcp.json`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CursorMcpConfig {
    #[serde(rename = "mcpServers", default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp_servers: BTreeMap<String, CursorMcpServer>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorMcpServer {
    #[serde(default = "default_stdio_type")]
    pub r#type: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
}

fn default_stdio_type() -> String {
    "stdio".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorRuleFrontmatter {
    pub description: String,
    pub globs: String,
    pub always_apply: bool,
}

impl Default for CursorRuleFrontmatter {
    fn default() -> Self {
        Self {
            description: "Compound Engineering Agent Directives".to_string(),
            globs: "*".to_string(),
            always_apply: true,
        }
    }
}
```

## 2. Functions & API Contract

1. `pub fn register_cursor_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`:
   - Reads existing `~/.cursor/mcp.json` (or creates new default).
   - Inserts `name` into `mcp_servers` map.
   - Writes atomically via `crate::state::write_atomic`.
2. `pub fn unregister_cursor_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError>`:
   - Removes `name` from `mcp_servers` map.
   - Saves atomically if modified. If `mcp_servers` and `extra` are empty and file was created by `ce-ai`, removes `mcp.json`. Backup restoration via `crate::state::backups` handles snapshot rollbacks.
3. `pub fn update_cursor_rule_mdc(rule_path: &Path, frontmatter: &CursorRuleFrontmatter, managed_content: &str) -> Result<(), CeError>`:
   - Writes or updates `.cursor/rules/compound-engineering.mdc` with frontmatter and demarcated block.

## 3. Sequence Flow

```
[ce-ai install --harness cursor]
  └─► Resolve home_dir.join(".cursor")
  └─► Ensure ~/.cursor/compound-engineering/ exists
  └─► Register sidecar MCP servers (codegraph, engram) into ~/.cursor/mcp.json under mcpServers key
  └─► Write workspace rule .cursor/rules/compound-engineering.mdc if in adopted project
  └─► Write manifest.json tracking managed files
  └─► Update state.json with installed_harnesses: ["cursor"]
```
