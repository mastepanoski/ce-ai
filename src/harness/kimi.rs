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
#[path = "tests/kimi.rs"]
mod tests;
