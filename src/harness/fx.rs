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
        if home.file_name().and_then(|n| n.to_str()) == Some(".fx") {
            return home.join("mcp.json");
        }
        self.kind().harness_dir(home).join("mcp.json")
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
            if let Err(e) = std::fs::remove_file(config_path) {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(e.into());
                }
            }
        } else {
            let json_bytes = serde_json::to_vec_pretty(&config)
                .map_err(|e| CeError::Runtime(format!("failed to serialize fx MCP config: {e}")))?;
            write_atomic(config_path, &json_bytes)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/fx.rs"]
mod tests;
