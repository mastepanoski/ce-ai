# Design: fx Native Harness Adapter

## System Architecture

### `FxAdapter` (Struct & `HarnessAdapter` Trait Implementation)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FxAdapter;

impl HarnessAdapter for FxAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Fx
    }

    fn default_config_path(&self, home_dir: &Path) -> PathBuf {
        self.kind().harness_dir(home_dir).join("mcp.json")
    }

    fn canonical_instruction_file(&self) -> PathBuf {
        PathBuf::from("AGENTS.md")
    }

    fn derived_stub_files(&self) -> Vec<PathBuf> {
        vec![PathBuf::from(".fx").join("AGENTS.md")]
    }
}
```

## Data Schemas

### `FxMcpConfig` (Serde Struct)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FxMcpConfig {
    #[serde(default)]
    pub mcp: BTreeMap<String, FxMcpServer>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

### `FxMcpServer` (Serde Struct)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FxMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub environment: BTreeMap<String, String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
```

## Helper Signatures & `HarnessKind` Dispatch
```rust
pub fn register_fx_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError>;

pub fn unregister_fx_mcp_server(
    config_path: &Path,
    name: &str,
) -> Result<(), CeError>;

// HarnessKind match arms:
// harness_dir(home_dir) => env("FX_HOME").unwrap_or(home_dir.join(".fx"))
// is_installed_on_host(home_dir) => harness_dir(home_dir).exists() || home_dir.join(".fx").exists()
// is_ce_installed(home_dir) => harness_dir.join("mcp.json").exists() || harness_dir.join("skills").exists()
```

## Environment Resolution & Directory Rules
1. If `$FX_HOME` is set and non-empty, use that path as `harness_dir`.
2. Else default to `<home_dir>/.fx`.
3. Config path is `harness_dir.join("mcp.json")`.
4. Skills directory is `harness_dir.join("skills")`.
5. Project rules target `AGENTS.md` (root) and `.fx/AGENTS.md` (when `.fx/` directory pre-exists).
