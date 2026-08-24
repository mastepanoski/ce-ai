//! `fx` native harness adapter implementation for Vercel Labs' fx coding agent (`~/.fx/`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

/// Harness adapter implementation for Vercel Labs' `fx` coding agent (`~/.fx/`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FxAdapter;

impl HarnessAdapter for FxAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Fx
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("mcp.json") {
            return home.to_path_buf();
        }
        if home.file_name().and_then(|n| n.to_str()) == Some(".fx")
            || home.join("mcp.json").exists()
        {
            return home.join("mcp.json");
        }
        self.kind().harness_dir(home).join("mcp.json")
    }

    fn canonical_instruction_file(&self) -> PathBuf {
        PathBuf::from("AGENTS.md")
    }

    fn derived_stub_files(&self) -> Vec<PathBuf> {
        vec![PathBuf::from(".fx").join("AGENTS.md")]
    }
}

/// Root structure of `~/.fx/mcp.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FxMcpConfig {
    #[serde(default)]
    pub mcp: BTreeMap<String, FxMcpServer>,

    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Server entry inside `~/.fx/mcp.json`.
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

/// Registers or updates an MCP server entry under root key `mcp` in `~/.fx/mcp.json`.
pub fn register_fx_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            FxMcpConfig::default()
        } else {
            serde_json::from_str::<FxMcpConfig>(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "invalid fx MCP config in {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        FxMcpConfig::default()
    };

    let mut full_command = vec![command.to_string()];
    full_command.extend(args.iter().map(|s| s.to_string()));

    let mut existing_extra = config.mcp.remove(name).map(|s| s.extra).unwrap_or_default();
    existing_extra.remove("type");

    let server_entry = FxMcpServer {
        r#type: Some("local".to_string()),
        command: full_command,
        environment: env.clone(),
        extra: existing_extra,
    };

    config.mcp.insert(name.to_string(), server_entry);

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json_bytes = serde_json::to_vec_pretty(&config)
        .map_err(|e| CeError::Runtime(format!("failed to serialize fx MCP config: {e}")))?;
    write_atomic(config_path, &json_bytes)?;

    Ok(())
}

/// Unregisters an MCP server entry from `~/.fx/mcp.json`.
pub fn unregister_fx_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    let mut config = serde_json::from_str::<FxMcpConfig>(&content).map_err(|e| {
        CeError::Runtime(format!(
            "invalid fx MCP config in {}: {e}",
            config_path.display()
        ))
    })?;

    if config.mcp.remove(name).is_some() {
        if config.mcp.is_empty() && config.extra.is_empty() {
            let _ = std::fs::remove_file(config_path);
        } else {
            let json_bytes = serde_json::to_vec_pretty(&config)
                .map_err(|e| CeError::Runtime(format!("failed to serialize fx MCP config: {e}")))?;
            write_atomic(config_path, &json_bytes)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::harness::tests::HARNESS_ENV_LOCK;

    #[test]
    fn fx_adapter_default_paths() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("FX_HOME");

        let adapter = FxAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Fx);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.kind().harness_dir(&home),
            PathBuf::from("/tmp/home/.fx")
        );
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.fx/mcp.json")
        );
        assert_eq!(
            adapter.default_config_path(&PathBuf::from("/tmp/home/.fx")),
            PathBuf::from("/tmp/home/.fx/mcp.json")
        );
        assert_eq!(
            adapter.default_config_path(&PathBuf::from("/tmp/home/.fx/mcp.json")),
            PathBuf::from("/tmp/home/.fx/mcp.json")
        );
        assert_eq!(
            adapter.canonical_instruction_file(),
            PathBuf::from("AGENTS.md")
        );
        assert_eq!(
            adapter.derived_stub_files(),
            vec![PathBuf::from(".fx/AGENTS.md")]
        );
    }

    #[test]
    fn fx_adapter_respects_fx_home_env() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::set_var("FX_HOME", "/custom/fx/dir");

        let adapter = FxAdapter;
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.kind().harness_dir(&home),
            PathBuf::from("/custom/fx/dir")
        );
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/custom/fx/dir/mcp.json")
        );

        std::env::remove_var("FX_HOME");
    }

    #[test]
    fn registers_and_unregisters_native_fx_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let mut env = BTreeMap::new();
        env.insert("KEY".to_string(), "VAL".to_string());

        register_fx_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"mcp\":"));
        assert!(content.contains("\"type\": \"local\""));
        assert!(content.contains("\"command\": ["));

        let config: FxMcpConfig = serde_json::from_str(&content).unwrap();

        assert!(config.mcp.contains_key("codegraph"));
        let server = &config.mcp["codegraph"];
        assert_eq!(server.r#type.as_deref(), Some("local"));
        assert_eq!(server.command, vec!["codegraph", "mcp"]);
        assert_eq!(
            server.environment.get("KEY").map(|s| s.as_str()),
            Some("VAL")
        );

        unregister_fx_mcp_server(&config_path, "codegraph").unwrap();
        assert!(!config_path.exists());
    }

    #[test]
    fn preserves_existing_user_fx_keys_and_extra_fields() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let initial_json = r#"{
            "user_setting": "active",
            "mcp": {
                "user_remote": {
                    "type": "http",
                    "url": "https://mcp.example.com"
                }
            }
        }"#;
        std::fs::write(&config_path, initial_json).unwrap();

        let env = BTreeMap::new();
        register_fx_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: FxMcpConfig = serde_json::from_str(&content).unwrap();

        assert_eq!(config.extra.get("user_setting").unwrap(), "active");
        assert!(config.mcp.contains_key("user_remote"));
        assert!(config.mcp.contains_key("codegraph"));

        unregister_fx_mcp_server(&config_path, "codegraph").unwrap();

        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: FxMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert!(config_after.mcp.contains_key("user_remote"));
        assert!(!config_after.mcp.contains_key("codegraph"));
    }
}
