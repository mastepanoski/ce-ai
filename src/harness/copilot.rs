//! GitHub Copilot AI harness adapter implementation.
//! Handles GitHub Copilot CLI / extension's native `~/.copilot/mcp-config.json` (`mcpServers` JSON schema)
//! and `.github/copilot-instructions.md` instruction file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

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

/// Native Copilot MCP server entry (`mcpServers.<name>` in JSON).
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

/// Native Copilot MCP configuration root file (`~/.copilot/mcp-config.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CopilotMcpConfig {
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: BTreeMap<String, CopilotMcpServer>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Merge and register an MCP server into Copilot's JSON config using native `mcpServers` schema.
pub fn register_copilot_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: CopilotMcpConfig = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            CopilotMcpConfig::default()
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Copilot mcp-config.json at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        CopilotMcpConfig::default()
    };

    let server_entry = config
        .mcp_servers
        .entry(name.to_string())
        .or_insert_with(|| CopilotMcpServer {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: env.clone(),
            extra: BTreeMap::new(),
        });

    server_entry.command = command.to_string();
    server_entry.args = args.iter().map(|s| s.to_string()).collect();
    if !env.is_empty() {
        for (k, v) in env {
            server_entry.env.insert(k.clone(), v.clone());
        }
    }

    let json_bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Copilot mcp-config.json at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, &json_bytes)
}

/// Unregister an MCP server from Copilot's JSON configuration file.
/// Removes the specified server entry from `mcpServers`. Leaves file intact to preserve user preferences.
pub fn unregister_copilot_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        return Ok(());
    }

    let mut config: CopilotMcpConfig = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Copilot mcp-config.json at {}: {e}",
            config_path.display()
        ))
    })?;

    config.mcp_servers.remove(name);

    let json_bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Copilot mcp-config.json at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, &json_bytes)
}

/// Write or update project directives in `.github/copilot-instructions.md` with demarcated managed block.
pub fn update_copilot_instructions_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError> {
    let existing_content = if rule_path.exists() {
        std::fs::read_to_string(rule_path)?
    } else {
        String::new()
    };

    let updated_body = update_managed_block(&existing_content, managed_text);
    write_atomic(rule_path, updated_body.as_bytes())
}

/// Inject or replace demarcated managed comment block in markdown instruction file.
pub fn update_managed_block(content: &str, managed_text: &str) -> String {
    let block = format!(
        "{}\n{}\n{}",
        CE_MANAGED_BEGIN,
        managed_text.trim(),
        CE_MANAGED_END
    );

    let start_opt = content.find(CE_MANAGED_BEGIN);
    let end_opt = content.find(CE_MANAGED_END);

    match (start_opt, end_opt) {
        (Some(start), Some(end)) if start <= end => {
            let before = content[..start].trim_end();
            let after = content[end + CE_MANAGED_END.len()..].trim_start();
            if before.is_empty() && after.is_empty() {
                block
            } else if before.is_empty() {
                format!("{}\n\n{}", block, after)
            } else if after.is_empty() {
                format!("{}\n\n{}", before, block)
            } else {
                format!("{}\n\n{}\n\n{}", before, block, after)
            }
        }
        (Some(start), _) => {
            let before = content[..start].trim_end();
            if before.is_empty() {
                block
            } else {
                format!("{}\n\n{}", before, block)
            }
        }
        (_, Some(end)) => {
            let after = content[end + CE_MANAGED_END.len()..].trim_start();
            if after.is_empty() {
                block
            } else {
                format!("{}\n\n{}", block, after)
            }
        }
        (None, None) => {
            if content.trim().is_empty() {
                block
            } else {
                format!("{}\n\n{}", content.trim_end(), block)
            }
        }
    }
}

/// Strip demarcated managed comment block on project de-adoption or uninstallation.
pub fn strip_managed_block(content: &str) -> String {
    let start_opt = content.find(CE_MANAGED_BEGIN);
    let end_opt = content.find(CE_MANAGED_END);

    match (start_opt, end_opt) {
        (Some(start), Some(end)) if start <= end => {
            let before = content[..start].trim_end();
            let after = content[end + CE_MANAGED_END.len()..].trim_start();
            if before.is_empty() {
                after.to_string()
            } else if after.is_empty() {
                before.to_string()
            } else {
                format!("{}\n\n{}", before, after)
            }
        }
        (Some(start), _) => content[..start].trim_end().to_string(),
        (_, Some(end)) => content[end + CE_MANAGED_END.len()..]
            .trim_start()
            .to_string(),
        (None, None) => content.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn copilot_adapter_default_paths() {
        let adapter = CopilotAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Copilot);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.copilot/mcp-config.json")
        );
    }

    #[test]
    fn registers_and_unregisters_native_copilot_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp-config.json");

        let mut env = BTreeMap::new();
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        register_copilot_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env)
            .unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
        assert!(config.mcp_servers.contains_key("codegraph"));
        assert_eq!(config.mcp_servers["codegraph"].command, "codegraph");
        assert_eq!(config.mcp_servers["codegraph"].args, vec!["mcp"]);
        assert!(config.extra.is_empty(), "Zero OpenCode key leaks");

        unregister_copilot_mcp_server(&config_path, "codegraph").unwrap();
        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: CopilotMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert!(!config_after.mcp_servers.contains_key("codegraph"));
    }

    #[test]
    fn preserves_existing_user_copilot_keys_and_extra_fields() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp-config.json");

        let initial_json = r#"{
  "telemetry": false,
  "mcpServers": {
    "user-tool": {
      "command": "node",
      "args": ["server.js"]
    }
  }
}"#;
        std::fs::write(&config_path, initial_json).unwrap();

        let env = BTreeMap::new();
        register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(
            config.extra.get("telemetry").unwrap(),
            &serde_json::Value::Bool(false)
        );
        assert!(config.mcp_servers.contains_key("user-tool"));
        assert!(config.mcp_servers.contains_key("engram"));

        unregister_copilot_mcp_server(&config_path, "engram").unwrap();

        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: CopilotMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert_eq!(
            config_after.extra.get("telemetry").unwrap(),
            &serde_json::Value::Bool(false)
        );
        assert!(config_after.mcp_servers.contains_key("user-tool"));
        assert!(!config_after.mcp_servers.contains_key("engram"));
    }

    #[test]
    fn updates_and_strips_copilot_instructions_md_managed_block() {
        let tmp = TempDir::new().unwrap();
        let md_path = tmp.path().join("copilot-instructions.md");

        let user_header = "# My Project Notes\n";
        std::fs::write(&md_path, user_header).unwrap();

        update_copilot_instructions_md(&md_path, "Directives content").unwrap();

        let content = std::fs::read_to_string(&md_path).unwrap();
        assert!(content.starts_with("# My Project Notes"));
        assert!(content.contains(CE_MANAGED_BEGIN));
        assert!(content.contains("Directives content"));
        assert!(content.contains(CE_MANAGED_END));

        let stripped = strip_managed_block(&content);
        assert!(!stripped.contains(CE_MANAGED_BEGIN));
        assert_eq!(stripped.trim(), "# My Project Notes");
    }

    #[test]
    fn register_copilot_mcp_server_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp-config.json");

        let env = BTreeMap::new();
        register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();
        register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert!(config.mcp_servers.contains_key("engram"));
    }
}
