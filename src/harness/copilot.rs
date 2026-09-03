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
    server_entry.env = env.clone();

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

pub const COPILOT_RESUME_COMMAND: &str = "ce-ai workflow resume --json";

/// Checks if `.github/hooks/hooks.json` contains a sessionStart hook executing `ce-ai workflow resume --json`.
pub fn has_session_start_hook(hooks_path: &Path) -> bool {
    if !hooks_path.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(hooks_path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    val.get("hooks")
        .and_then(|h| h.get("sessionStart"))
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter().any(|entry| {
                entry.get("bash").and_then(|b| b.as_str()) == Some(COPILOT_RESUME_COMMAND)
                    || entry.get("powershell").and_then(|p| p.as_str())
                        == Some(COPILOT_RESUME_COMMAND)
                    || entry.get("command").and_then(|c| c.as_str()) == Some(COPILOT_RESUME_COMMAND)
            })
        })
        .unwrap_or(false)
}

/// Ensures `.github/hooks/hooks.json` contains the sessionStart hook for `ce-ai workflow resume --json`.
/// Preserves any pre-existing user hooks or extra settings. Idempotent.
pub fn ensure_session_start_hook(hooks_path: &Path) -> Result<bool, CeError> {
    if has_session_start_hook(hooks_path) {
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

    if !root_obj.contains_key("version") {
        root_obj.insert("version".to_string(), serde_json::json!(1));
    }

    let hooks_val = root_obj
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    if !hooks_val.is_object() {
        *hooks_val = serde_json::json!({});
    }

    let hooks_obj = hooks_val
        .as_object_mut()
        .ok_or_else(|| CeError::Runtime("hooks is not an object".to_string()))?;

    let session_start_val = hooks_obj
        .entry("sessionStart")
        .or_insert_with(|| serde_json::json!([]));
    if !session_start_val.is_array() {
        *session_start_val = serde_json::json!([]);
    }

    let session_start_arr = session_start_val
        .as_array_mut()
        .ok_or_else(|| CeError::Runtime("sessionStart is not an array".to_string()))?;

    let target_hook = serde_json::json!({
        "type": "command",
        "bash": COPILOT_RESUME_COMMAND,
        "powershell": COPILOT_RESUME_COMMAND,
        "timeoutSec": 15
    });

    if !session_start_arr.iter().any(|entry| {
        entry.get("bash").and_then(|b| b.as_str()) == Some(COPILOT_RESUME_COMMAND)
            || entry.get("powershell").and_then(|p| p.as_str()) == Some(COPILOT_RESUME_COMMAND)
            || entry.get("command").and_then(|c| c.as_str()) == Some(COPILOT_RESUME_COMMAND)
    }) {
        session_start_arr.push(target_hook);
    }

    if let Some(parent) = hooks_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| CeError::Runtime(format!("failed to serialize hooks.json: {e}")))?;
    write_atomic(hooks_path, serialized.as_bytes())?;
    Ok(true)
}

/// Surgically removes `ce-ai workflow resume --json` hook from `.github/hooks/hooks.json`.
/// If the file becomes empty `{}` or only contains `{"version": 1}` as a result, removes the file cleanly.
pub fn remove_session_start_hook(hooks_path: &Path) -> Result<bool, CeError> {
    if !hooks_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(hooks_path)?;
    let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Ok(false);
    };

    let mut changed = false;

    if let Some(hooks_obj) = root.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        if let Some(session_start_arr) = hooks_obj
            .get_mut("sessionStart")
            .and_then(|s| s.as_array_mut())
        {
            let prev_len = session_start_arr.len();
            session_start_arr.retain(|entry| {
                entry.get("bash").and_then(|b| b.as_str()) != Some(COPILOT_RESUME_COMMAND)
                    && entry.get("powershell").and_then(|p| p.as_str())
                        != Some(COPILOT_RESUME_COMMAND)
                    && entry.get("command").and_then(|c| c.as_str()) != Some(COPILOT_RESUME_COMMAND)
            });
            if session_start_arr.len() != prev_len {
                changed = true;
            }

            if session_start_arr.is_empty() {
                hooks_obj.remove("sessionStart");
                changed = true;
            }
        }

        if hooks_obj.is_empty() {
            if let Some(root_obj) = root.as_object_mut() {
                root_obj.remove("hooks");
                changed = true;
            }
        }
    }

    if !changed {
        return Ok(false);
    }

    let is_empty_or_only_version = if let Some(obj) = root.as_object() {
        obj.is_empty() || (obj.len() == 1 && obj.contains_key("version"))
    } else {
        false
    };

    if is_empty_or_only_version {
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

#[cfg(test)]
#[path = "tests/copilot.rs"]
mod tests;
