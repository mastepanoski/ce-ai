# Design: Kimi Code CLI Native Harness Adapter (Issue #178)

## 1. KimiAdapter (`src/harness/kimi.rs`)

```rust
#[derive(Debug, Default)]
pub struct KimiAdapter;

impl HarnessAdapter for KimiAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Kimi
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("mcp.json") {
            return home.to_path_buf();
        }

        if let Some(config_env) = std::env::var_os("KIMI_CODE_HOME") {
            return PathBuf::from(config_env).join("mcp.json");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".kimi-code") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".kimi-code").join("mcp.json")
    }
}
```

## 2. Data Structures & Schemas (`src/harness/kimi.rs`)

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct KimiMcpConfig {
    #[serde(default, rename = "mcpServers", skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp_servers: BTreeMap<String, KimiMcpServer>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct KimiMcpServer {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}
```

## 3. Functions

1. `register_kimi_mcp_server(config_path, name, command, args, env) -> Result<(), CeError>`
   - Reads existing `mcp.json` or creates default struct.
   - Inserts/updates `mcp_servers[name]` with `KimiMcpServer`.
   - Serializes and writes back atomically using `crate::state::write_atomic`.

2. `unregister_kimi_mcp_server(config_path, name) -> Result<(), CeError>`
   - Removes `mcp_servers[name]`.
   - Writes back atomically using `crate::state::write_atomic`.

## 4. Environment Variable Overrides & Project Rules
- `$KIMI_CODE_HOME`: Custom home directory override for Kimi Code CLI (defaults to `$HOME/.kimi-code`). Note: `$KIMI_CODE_HOME` is official Kimi Code CLI environment variable convention.
- Project rule adoption targets project `AGENTS.md` with `CE-AI MANAGED BLOCK`.
