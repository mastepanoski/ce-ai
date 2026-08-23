# Design: Claude Code Native Harness Adapter (Issue #174)

## Structural Data Models (`src/harness/claude.rs`)

```rust
#[derive(Debug, Default)]
pub struct ClaudeAdapter;

impl HarnessAdapter for ClaudeAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Claude
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        let settings_path = home.join(".claude").join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if content.contains("\"mcpServers\"") {
                    return settings_path;
                }
            }
        }
        home.join(".claude.json")
    }
}

/// Root schema for Claude Code's user configuration (`~/.claude.json` / `~/.claude/settings.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClaudeMcpConfig {
    #[serde(
        rename = "mcpServers",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub mcp_servers: BTreeMap<String, ClaudeMcpServer>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Native Claude Code MCP server entry (stdio or SSE/http transport).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeMcpServer {
    #[serde(default = "default_stdio_type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

fn default_stdio_type() -> String {
    "stdio".to_string()
}
```

## Functions & Lifecycle Wiring

1. `register_claude_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`:
   - Reads `config_path` if present; parses `ClaudeMcpConfig`.
   - Inserts or updates `mcp_servers[name]`.
   - Writes back atomically using `write_atomic`.

2. `unregister_claude_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError>`:
   - Removes `name` from `mcp_servers`.
   - Deletes file ONLY if `mcp_servers` and `extra` are empty; otherwise writes updated JSON atomically.

3. `update_claude_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError>`:
   - Priority path resolution:
     1. `./CLAUDE.md` if present
     2. `.claude/CLAUDE.md` if `.claude/` directory exists
     3. Default to `./CLAUDE.md`
   - Updates demarcated `CE-AI MANAGED BLOCK`.

4. **Managed Skills Directory**:
   - Skills are installed under `~/.claude/skills/<name>/SKILL.md` (Agent Skills format).
