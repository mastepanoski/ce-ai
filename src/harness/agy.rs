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
#[path = "tests/agy.rs"]
mod tests;
