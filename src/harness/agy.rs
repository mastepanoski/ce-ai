use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Root configuration structure for Google Antigravity (`~/.gemini/config/mcp_config.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AgyMcpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: BTreeMap<String, AgyMcpServer>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Server entry structure for Google Antigravity MCP configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgyMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,

    #[serde(
        rename = "serverUrl",
        alias = "url",
        skip_serializing_if = "Option::is_none"
    )]
    pub server_url: Option<String>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Native HarnessAdapter implementation for Google Antigravity (`agy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AgyAdapter;

impl AgyAdapter {
    pub fn harness_dir(&self, home_dir: &Path) -> PathBuf {
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
}

impl HarnessAdapter for AgyAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Agy
    }

    fn default_config_path(&self, base_dir: &Path) -> PathBuf {
        if base_dir.file_name().and_then(|n| n.to_str()) == Some("mcp_config.json") {
            return base_dir.to_path_buf();
        }

        if base_dir.file_name().and_then(|n| n.to_str()) == Some("config") {
            return base_dir.join("mcp_config.json");
        }

        let home_dir = if base_dir.file_name().and_then(|n| n.to_str()) == Some(".gemini") {
            base_dir.parent().unwrap_or(base_dir)
        } else {
            base_dir
        };

        self.harness_dir(home_dir)
            .join("config")
            .join("mcp_config.json")
    }

    fn canonical_instruction_file(&self) -> PathBuf {
        PathBuf::from("GEMINI.md")
    }

    fn derived_stub_files(&self) -> Vec<PathBuf> {
        vec![PathBuf::from(".agents")
            .join("rules")
            .join("compound-engineering.md")]
    }
}

/// Merge and register an MCP server into Google Antigravity's `mcp_config.json` config using native `mcpServers` JSON object schema.
pub fn register_agy_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: AgyMcpConfig = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            AgyMcpConfig::default()
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Google Antigravity mcp_config.json at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        AgyMcpConfig::default()
    };

    let server_entry = config
        .mcp_servers
        .entry(name.to_string())
        .or_insert_with(|| AgyMcpServer {
            command: Some(command.to_string()),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: env.clone(),
            server_url: None,
            extra: serde_json::Map::new(),
        });

    server_entry.command = Some(command.to_string());
    server_entry.args = args.iter().map(|s| s.to_string()).collect();
    server_entry.env = env.clone();
    server_entry.server_url = None;
    for key in ["url", "serverUrl", "headers", "transport"] {
        server_entry.extra.remove(key);
    }

    let updated_json = serde_json::to_string_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Google Antigravity mcp_config.json: {e}"
        ))
    })?;

    write_atomic(config_path, updated_json.as_bytes())
}

/// Remove an MCP server from Google Antigravity's `mcp_config.json` config.
pub fn unregister_agy_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;

    if content.trim().is_empty() {
        return Ok(());
    }

    let mut config: AgyMcpConfig = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Google Antigravity mcp_config.json at {}: {e}",
            config_path.display()
        ))
    })?;

    config.mcp_servers.remove(name);

    let updated_json = serde_json::to_string_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Google Antigravity mcp_config.json: {e}"
        ))
    })?;

    write_atomic(config_path, updated_json.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::tests::HARNESS_ENV_LOCK;
    use tempfile::TempDir;

    #[test]
    fn agy_adapter_default_paths() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("ANTIGRAVITY_CONFIG_DIR");
        std::env::remove_var("GEMINI_HOME");

        let adapter = AgyAdapter;
        let home = Path::new("/tmp/home");

        assert_eq!(adapter.kind(), HarnessKind::Agy);
        assert_eq!(adapter.harness_dir(home), home.join(".gemini"));
        assert_eq!(
            adapter.default_config_path(home),
            home.join(".gemini/config/mcp_config.json")
        );
        assert_eq!(
            adapter.canonical_instruction_file(),
            PathBuf::from("GEMINI.md")
        );
        assert_eq!(
            adapter.derived_stub_files(),
            vec![PathBuf::from(".agents/rules/compound-engineering.md")]
        );
    }

    #[test]
    fn agy_adapter_respects_env_overrides() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::set_var("ANTIGRAVITY_CONFIG_DIR", "/custom/agy/dir");

        let adapter = AgyAdapter;
        let home = Path::new("/tmp/home");

        assert_eq!(adapter.harness_dir(home), PathBuf::from("/custom/agy/dir"));
        assert_eq!(
            adapter.default_config_path(home),
            PathBuf::from("/custom/agy/dir/config/mcp_config.json")
        );

        std::env::set_var("GEMINI_HOME", "/custom/gemini/dir");
        assert_eq!(
            adapter.harness_dir(home),
            PathBuf::from("/custom/agy/dir"),
            "ANTIGRAVITY_CONFIG_DIR takes precedence over GEMINI_HOME"
        );

        std::env::remove_var("ANTIGRAVITY_CONFIG_DIR");

        assert_eq!(
            adapter.harness_dir(home),
            PathBuf::from("/custom/gemini/dir")
        );
        assert_eq!(
            adapter.default_config_path(home),
            PathBuf::from("/custom/gemini/dir/config/mcp_config.json")
        );
        std::env::remove_var("GEMINI_HOME");
    }

    #[test]
    fn register_and_unregister_agy_mcp_server_preserves_remote_server_url() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp_config.json");

        // Seed with a remote server using serverUrl
        let initial_json = r#"{
          "custom_root_key": "active",
          "mcpServers": {
            "remote_server": {
              "serverUrl": "https://mcp.example.com/sse",
              "headers": { "Auth": "Bearer token" }
            }
          }
        }"#;
        std::fs::write(&config_path, initial_json).unwrap();

        let mut env = BTreeMap::new();
        env.insert("KEY".to_string(), "VAL".to_string());

        register_agy_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: AgyMcpConfig = serde_json::from_str(&content).unwrap();

        assert_eq!(
            config
                .extra
                .get("custom_root_key")
                .unwrap()
                .as_str()
                .unwrap(),
            "active"
        );

        assert!(config.mcp_servers.contains_key("codegraph"));
        let codegraph = &config.mcp_servers["codegraph"];
        assert_eq!(codegraph.command.as_deref(), Some("codegraph"));
        assert_eq!(codegraph.args, vec!["mcp"]);
        assert_eq!(codegraph.env.get("KEY").unwrap(), "VAL");

        assert!(config.mcp_servers.contains_key("remote_server"));
        let remote = &config.mcp_servers["remote_server"];
        assert_eq!(
            remote.server_url.as_deref(),
            Some("https://mcp.example.com/sse")
        );

        unregister_agy_mcp_server(&config_path, "codegraph").unwrap();

        let after_unreg = std::fs::read_to_string(&config_path).unwrap();
        let config_after: AgyMcpConfig = serde_json::from_str(&after_unreg).unwrap();
        assert!(!config_after.mcp_servers.contains_key("codegraph"));
        assert!(config_after.mcp_servers.contains_key("remote_server"));
    }

    #[test]
    fn register_agy_mcp_server_resets_server_url_on_name_collision() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp_config.json");

        // Seed with codegraph entry containing serverUrl and url alias
        let initial_json = r#"{
          "mcpServers": {
            "codegraph": {
              "url": "https://mcp.example.com/codegraph",
              "headers": { "Auth": "Bearer token" }
            },
            "other_remote": {
              "serverUrl": "https://mcp.example.com/other"
            }
          }
        }"#;
        std::fs::write(&config_path, initial_json).unwrap();

        let env = BTreeMap::new();
        register_agy_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json_val
            .pointer("/mcpServers/codegraph/serverUrl")
            .is_none());
        assert!(json_val.pointer("/mcpServers/codegraph/url").is_none());
        assert!(json_val.pointer("/mcpServers/codegraph/headers").is_none());

        let config: AgyMcpConfig = serde_json::from_str(&content).unwrap();

        let codegraph = &config.mcp_servers["codegraph"];
        assert_eq!(codegraph.server_url, None);
        assert_eq!(codegraph.command.as_deref(), Some("codegraph"));

        let other = &config.mcp_servers["other_remote"];
        assert_eq!(
            other.server_url.as_deref(),
            Some("https://mcp.example.com/other")
        );
    }

    #[test]
    fn register_agy_mcp_server_excludes_opencode_keys() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp_config.json");
        let env = BTreeMap::new();

        register_agy_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json_val.get("plugin").is_none());
        assert!(json_val.get("skills").is_none());
        assert!(!content.contains("plugin"));
        assert!(!content.contains("skills"));
    }
}
