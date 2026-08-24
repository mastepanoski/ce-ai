# Design: Google Antigravity (agy) Native Harness Adapter

## System Architecture

### `AgyAdapter` (Struct & `HarnessAdapter` Trait Implementation)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AgyAdapter;

impl HarnessAdapter for AgyAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Agy
    }

    fn harness_dir(&self, home_dir: &Path) -> PathBuf {
        if let Ok(path) = std::env::var("ANTIGRAVITY_CONFIG_DIR") {
            if !path.trim().is_empty() {
                return PathBuf::from(path);
            }
        }
        if let Ok(path) = std::env::var("GEMINI_HOME") {
            if !path.trim().is_empty() {
                return PathBuf::from(path);
            }
        }
        home_dir.join(".gemini")
    }

    fn default_config_path(&self, home_dir: &Path) -> PathBuf {
        self.harness_dir(home_dir).join("config").join("mcp_config.json")
    }

    fn canonical_instruction_file(&self, project_dir: &Path) -> PathBuf {
        project_dir.join("GEMINI.md")
    }

    fn derived_stub_files(&self, project_dir: &Path) -> Vec<PathBuf> {
        vec![project_dir.join(".agents").join("rules").join("compound-engineering.md")]
    }

    fn register_mcp_server(
        &self,
        config_path: &Path,
        name: &str,
        command: &str,
        args: &[&str],
        env: &BTreeMap<String, String>,
    ) -> Result<(), CeError> {
        register_agy_mcp_server(config_path, name, command, args, env)
    }

    fn unregister_mcp_server(&self, config_path: &Path, name: &str) -> Result<(), CeError> {
        unregister_agy_mcp_server(config_path, name)
    }
}
```

## Data Schemas

### `AgyMcpConfig` (Serde Struct)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AgyMcpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: BTreeMap<String, AgyMcpServer>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

### `AgyMcpServer` (Serde Struct)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgyMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,

    #[serde(rename = "serverUrl", skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

## Functions in `src/harness/agy.rs`
- `pub fn register_agy_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`
- `pub fn unregister_agy_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError>`

## Environment Resolution & Backup Matching Rules
1. If `$ANTIGRAVITY_CONFIG_DIR` is set and non-empty, use that directory as `harness_dir`. (Note: `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` are `ce-ai` extension conventions for custom directory relocation).
2. Else if `$GEMINI_HOME` is set and non-empty, use that directory as `harness_dir`.
3. Else default to `<home_dir>/.gemini`.
4. `config_path` is `harness_dir.join("config").join("mcp_config.json")`.
5. Skills directory is `harness_dir.join("config").join("skills")`.
6. Backup filter in `src/state/backups.rs` matches entries containing `"gemini"`, `"antigravity"`, or `"agy"`.

## Server Registration Name Collision Policy
- When `register_agy_mcp_server` registers a server entry under a name that previously had a remote `serverUrl` definition, `command`, `args`, and `env` are updated, and `server_url` is explicitly set to `None` to convert the entry cleanly to a stdio command server.
- Remote server entries with distinct names (e.g. `serverUrl: "https://..."`) are preserved intact alongside native stdio tools.

## Project Rules Architecture
- Canonical instruction file is `<project_dir>/GEMINI.md`.
- Derived stub rule file is `<project_dir>/.agents/rules/compound-engineering.md` (adopted when `.agents/` directory pre-exists).
