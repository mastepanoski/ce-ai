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

pub const AGY_RESUME_COMMAND: &str = "ce-ai workflow resume --pre-invocation";

/// Checks if `.agents/hooks.json` (or any hooks configuration) contains the PreInvocation hook for `ce-ai workflow resume --pre-invocation`.
pub fn has_pre_invocation_hook(hooks_path: &Path) -> bool {
    if !hooks_path.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(hooks_path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    let Some(obj) = val.as_object() else {
        return false;
    };
    obj.values().any(|group| {
        group
            .get("PreInvocation")
            .and_then(|pi| pi.as_array())
            .map(|arr| {
                arr.iter().any(|entry| {
                    entry.get("command").and_then(|c| c.as_str()) == Some(AGY_RESUME_COMMAND)
                })
            })
            .unwrap_or(false)
    })
}

/// Ensures `.agents/hooks.json` contains the PreInvocation hook for `ce-ai workflow resume --pre-invocation`.
/// Preserves any pre-existing user hooks or extra hook groups. Idempotent.
pub fn ensure_pre_invocation_hook(hooks_path: &Path) -> Result<bool, CeError> {
    if has_pre_invocation_hook(hooks_path) {
        return Ok(false);
    }

    let mut root: serde_json::Value = if hooks_path.exists() {
        let content = std::fs::read_to_string(hooks_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !root.is_object() {
        root = serde_json::json!({});
    }

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| CeError::Runtime("hooks root is not an object".to_string()))?;

    let ce_val = root_obj
        .entry("compound-engineering")
        .or_insert_with(|| serde_json::json!({}));
    if !ce_val.is_object() {
        *ce_val = serde_json::json!({});
    }

    let ce_obj = ce_val.as_object_mut().ok_or_else(|| {
        CeError::Runtime("compound-engineering hook group is not an object".to_string())
    })?;

    let pre_inv_val = ce_obj
        .entry("PreInvocation")
        .or_insert_with(|| serde_json::json!([]));
    if !pre_inv_val.is_array() {
        *pre_inv_val = serde_json::json!([]);
    }

    let pre_inv_arr = pre_inv_val
        .as_array_mut()
        .ok_or_else(|| CeError::Runtime("PreInvocation is not an array".to_string()))?;

    let target_hook = serde_json::json!({
        "type": "command",
        "command": AGY_RESUME_COMMAND,
    });

    if !pre_inv_arr
        .iter()
        .any(|entry| entry.get("command").and_then(|c| c.as_str()) == Some(AGY_RESUME_COMMAND))
    {
        pre_inv_arr.push(target_hook);
    }

    if let Some(parent) = hooks_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| CeError::Runtime(format!("failed to serialize hooks.json: {e}")))?;
    write_atomic(hooks_path, serialized.as_bytes())?;
    Ok(true)
}

/// Surgically removes `ce-ai workflow resume --pre-invocation` hook from `.agents/hooks.json`.
/// If the file becomes effectively empty as a result, removes the file cleanly and prunes the parent directory if empty.
pub fn remove_pre_invocation_hook(hooks_path: &Path) -> Result<bool, CeError> {
    if !hooks_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(hooks_path)?;
    let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Ok(false);
    };

    let Some(root_obj) = root.as_object_mut() else {
        return Ok(false);
    };

    let mut changed = false;
    let mut empty_groups = Vec::new();

    for (group_name, group_val) in root_obj.iter_mut() {
        if let Some(group_obj) = group_val.as_object_mut() {
            if let Some(pre_inv_arr) = group_obj
                .get_mut("PreInvocation")
                .and_then(|pi| pi.as_array_mut())
            {
                let orig_len = pre_inv_arr.len();
                pre_inv_arr.retain(|entry| {
                    entry.get("command").and_then(|c| c.as_str()) != Some(AGY_RESUME_COMMAND)
                });
                if pre_inv_arr.len() != orig_len {
                    changed = true;
                }
                if pre_inv_arr.is_empty() {
                    group_obj.remove("PreInvocation");
                }
            }
            if group_obj.is_empty() {
                empty_groups.push(group_name.clone());
            }
        }
    }

    for group in empty_groups {
        root_obj.remove(&group);
    }

    if !changed {
        return Ok(false);
    }

    if root_obj.is_empty() {
        let _ = std::fs::remove_file(hooks_path);
        if let Some(parent) = hooks_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
        return Ok(true);
    }

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| CeError::Runtime(format!("failed to serialize hooks.json: {e}")))?;
    write_atomic(hooks_path, serialized.as_bytes())?;
    Ok(true)
}

// Backward-compatible alias helpers
pub use ensure_pre_invocation_hook as ensure_session_start_hook;
pub use has_pre_invocation_hook as has_session_start_hook;
pub use remove_pre_invocation_hook as remove_session_start_hook;

#[cfg(test)]
#[path = "tests/agy.rs"]
mod tests;
