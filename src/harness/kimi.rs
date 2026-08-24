//! Kimi Code CLI AI harness adapter implementation.
//! Handles Kimi Code CLI's native `~/.kimi-code/mcp.json` (`mcpServers` JSON object schema)
//! and `AGENTS.md` instruction file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct KimiMcpConfig {
    #[serde(
        default,
        rename = "mcpServers",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
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

/// Merge and register an MCP server into Kimi Code CLI's `mcp.json` config using native `mcpServers` JSON object schema.
pub fn register_kimi_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: KimiMcpConfig = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            KimiMcpConfig::default()
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Kimi mcp.json at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        KimiMcpConfig::default()
    };

    let server_entry = config
        .mcp_servers
        .entry(name.to_string())
        .or_insert_with(|| KimiMcpServer {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: env.clone(),
            extra: serde_json::Map::new(),
        });

    server_entry.command = command.to_string();
    server_entry.args = args.iter().map(|s| s.to_string()).collect();
    server_entry.env = env.clone();

    let updated_json = serde_json::to_string_pretty(&config)
        .map_err(|e| CeError::Runtime(format!("Failed to serialize Kimi mcp.json: {e}")))?;

    write_atomic(config_path, updated_json.as_bytes())
}

/// Remove an MCP server from Kimi Code CLI's `mcp.json` config.
pub fn unregister_kimi_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;

    if content.trim().is_empty() {
        return Ok(());
    }

    let mut config: KimiMcpConfig = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Kimi mcp.json at {}: {e}",
            config_path.display()
        ))
    })?;

    config.mcp_servers.remove(name);

    let updated_json = serde_json::to_string_pretty(&config)
        .map_err(|e| CeError::Runtime(format!("Failed to serialize Kimi mcp.json: {e}")))?;

    write_atomic(config_path, updated_json.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn kimi_adapter_default_paths() {
        let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("KIMI_CODE_HOME");
        let adapter = KimiAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Kimi);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.kimi-code/mcp.json")
        );
    }

    #[test]
    fn kimi_adapter_respects_kimi_code_home_env() {
        let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
        let adapter = KimiAdapter;
        let home = PathBuf::from("/tmp/home");
        std::env::set_var("KIMI_CODE_HOME", "/custom/kimi/dir");
        let path = adapter.default_config_path(&home);
        std::env::remove_var("KIMI_CODE_HOME");
        assert_eq!(path, PathBuf::from("/custom/kimi/dir/mcp.json"));
    }

    #[test]
    fn registers_and_unregisters_native_kimi_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let mut env = BTreeMap::new();
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        register_kimi_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: KimiMcpConfig = serde_json::from_str(&content).unwrap();
        assert!(config.mcp_servers.contains_key("codegraph"));

        let codegraph = &config.mcp_servers["codegraph"];
        assert_eq!(codegraph.command, "codegraph");
        assert_eq!(codegraph.args, vec!["mcp"]);
        assert_eq!(codegraph.env.get("LOG_LEVEL").unwrap(), "info");

        // Verify zero OpenCode keys leak into JSON
        assert!(!content.contains("plugin"));
        assert!(!content.contains("skills.paths"));

        // Unregister
        unregister_kimi_mcp_server(&config_path, "codegraph").unwrap();
        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: KimiMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert!(!config_after.mcp_servers.contains_key("codegraph"));
    }

    #[test]
    fn replaces_env_map_cleanly_on_re_registration() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let mut env1 = BTreeMap::new();
        env1.insert("OLD_KEY".to_string(), "old_val".to_string());
        register_kimi_mcp_server(&config_path, "engram", "engram", &["serve"], &env1).unwrap();

        let mut env2 = BTreeMap::new();
        env2.insert("NEW_KEY".to_string(), "new_val".to_string());
        register_kimi_mcp_server(&config_path, "engram", "engram", &["serve"], &env2).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: KimiMcpConfig = serde_json::from_str(&content).unwrap();
        let engram_env = &config.mcp_servers["engram"].env;
        assert!(!engram_env.contains_key("OLD_KEY"));
        assert_eq!(engram_env.get("NEW_KEY").unwrap(), "new_val");

        // Re-register with empty env map -> removes env key from JSON
        let empty_env = BTreeMap::new();
        register_kimi_mcp_server(&config_path, "engram", "engram", &["serve"], &empty_env).unwrap();
        let content_empty = std::fs::read_to_string(&config_path).unwrap();
        assert!(!content_empty.contains("\"env\""));

        let config_empty: KimiMcpConfig = serde_json::from_str(&content_empty).unwrap();
        assert!(config_empty.mcp_servers["engram"].env.is_empty());
    }
}
