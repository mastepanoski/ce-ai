//! opencode.json read → merge (dedup) → atomic write; hard-fails on invalid
//! existing JSON instead of clobbering user config (OI-2, D4).

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::state::{ConfigStore, FsConfigStore};

/// A config mutation recorded in the install manifest (OI-5, design §Interfaces).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigMutation {
    pub file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backup: Option<String>,
    pub keys: Vec<String>,
}

/// Reads a harness JSON config. Kept as a thin alias over the neutral
/// `state::read_config` for opencode-specific callers.
pub use crate::state::read_config;

/// Appends `plugin_entry` to `plugin` unless already present (OI-2). Fails
/// instead of clobbering when `plugin` exists but is not an array.
fn merge_plugin(config: &mut serde_json::Value, plugin_entry: &str) -> Result<(), CeError> {
    match config.get_mut("plugin") {
        None => config["plugin"] = serde_json::json!([plugin_entry]),
        Some(serde_json::Value::Array(arr)) => {
            if !arr.iter().any(|v| v.as_str() == Some(plugin_entry)) {
                arr.push(serde_json::Value::String(plugin_entry.to_string()));
            }
        }
        Some(_) => {
            return Err(CeError::Runtime(
                "`plugin` in opencode.json must be an array; refusing to overwrite it. Fix the file manually, then re-run."
                    .into(),
            ))
        }
    }
    Ok(())
}

/// Appends `skills_path` to `skills.paths` unless already present (OI-4). Fails
/// instead of clobbering malformed `skills`/`skills.paths` values.
fn merge_skills_path(config: &mut serde_json::Value, skills_path: &str) -> Result<(), CeError> {
    match config.get_mut("skills") {
        None => config["skills"] = serde_json::json!({ "paths": [skills_path] }),
        Some(serde_json::Value::Object(skills)) => match skills.get_mut("paths") {
            None => {
                skills.insert("paths".into(), serde_json::json!([skills_path]));
            }
            Some(serde_json::Value::Array(paths)) => {
                if !paths.iter().any(|v| v.as_str() == Some(skills_path)) {
                    paths.push(serde_json::Value::String(skills_path.to_string()));
                }
            }
            Some(_) => {
                return Err(CeError::Runtime(
                    "`skills.paths` in opencode.json must be an array; refusing to overwrite it. Fix the file manually, then re-run."
                        .into(),
                ))
            }
        },
        Some(_) => {
            return Err(CeError::Runtime(
                "`skills` in opencode.json must be an object; refusing to overwrite it. Fix the file manually, then re-run."
                    .into(),
            ))
        }
    }
    Ok(())
}

/// Returns the mutation record for the install manifest using an explicit `ConfigStore` (KTD4).
pub fn ensure_plugin_and_skills_with_store(
    store: &dyn ConfigStore,
    config_path: &Path,
    plugin_entry: &str,
    skills_path: &str,
) -> Result<ConfigMutation, CeError> {
    let mut config = store.read_config(config_path)?;
    merge_plugin(&mut config, plugin_entry)?;
    merge_skills_path(&mut config, skills_path)?;
    store.write_config(config_path, &config)?;
    Ok(ConfigMutation {
        file: config_path.display().to_string(),
        backup: None,
        keys: vec!["plugin".into(), "skills.paths".into()],
    })
}

/// Returns the mutation record for the install manifest (OI-5).
pub fn ensure_plugin_and_skills(
    config_path: &Path,
    plugin_entry: &str,
    skills_path: &str,
) -> Result<ConfigMutation, CeError> {
    ensure_plugin_and_skills_with_store(&FsConfigStore, config_path, plugin_entry, skills_path)
}

/// Merges an MCP server definition into `opencode.json` using an explicit `ConfigStore` (KTD4).
pub fn register_mcp_server_with_store(
    store: &dyn ConfigStore,
    config_path: &Path,
    tool_name: &str,
    server_def: serde_json::Value,
) -> Result<(), CeError> {
    let mut config = store.read_config(config_path)?;
    if !config.is_object() {
        return Err(CeError::Runtime(format!(
            "{} must be a JSON object; refusing to overwrite it. Fix the file manually, then re-run.",
            config_path.display()
        )));
    }
    match config.get_mut("mcpServers") {
        None => {
            config["mcpServers"] = serde_json::json!({
                tool_name: server_def
            });
        }
        Some(serde_json::Value::Object(mcp)) => {
            mcp.insert(tool_name.to_string(), server_def);
        }
        Some(_) => {
            return Err(CeError::Runtime(
                "`mcpServers` in opencode.json must be an object; refusing to overwrite it. Fix the file manually, then re-run."
                    .into(),
            ));
        }
    }
    store.write_config(config_path, &config)?;
    Ok(())
}

/// Merges an MCP server definition into `opencode.json` under `mcpServers.<tool_name>`.
/// Preserves pre-existing user MCP servers and custom config. Writes atomically.
pub fn register_mcp_server(
    config_path: &Path,
    tool_name: &str,
    server_def: serde_json::Value,
) -> Result<(), CeError> {
    register_mcp_server_with_store(&FsConfigStore, config_path, tool_name, server_def)
}

#[cfg(test)]
#[path = "tests/config.rs"]
mod tests;
