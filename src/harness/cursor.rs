//! Cursor native harness adapter implementation.
//! Handles Cursor's native `~/.cursor/mcp.json` (`mcpServers` stdio schema)
//! and `.cursor/rules/*.mdc` instruction files.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::state::write_atomic;

pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

/// Root schema for Cursor's `~/.cursor/mcp.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CursorMcpConfig {
    #[serde(
        rename = "mcpServers",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub mcp_servers: BTreeMap<String, CursorMcpServer>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Native Cursor MCP server entry (`type: stdio` or SSE).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorMcpServer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Frontmatter header for `.cursor/rules/*.mdc` project rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorRuleFrontmatter {
    pub description: String,
    pub globs: String,
    pub always_apply: bool,
}

impl Default for CursorRuleFrontmatter {
    fn default() -> Self {
        Self {
            description: "Compound Engineering Agent Directives".to_string(),
            globs: "*".to_string(),
            always_apply: true,
        }
    }
}

/// Merge and register an MCP server into `~/.cursor/mcp.json` using Cursor's native `mcpServers` schema.
pub fn register_cursor_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: CursorMcpConfig = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            CursorMcpConfig::default()
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Cursor mcp.json at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        CursorMcpConfig::default()
    };

    let mut server = config
        .mcp_servers
        .remove(name)
        .unwrap_or_else(|| CursorMcpServer {
            r#type: Some("stdio".to_string()),
            command: String::new(),
            args: Vec::new(),
            env: BTreeMap::new(),
            url: None,
            extra: serde_json::Map::new(),
        });

    if server.r#type.is_none() {
        server.r#type = Some("stdio".to_string());
    }
    if server.r#type.as_deref() == Some("stdio") {
        server.url = None;
    }
    server.command = command.to_string();
    server.args = args.iter().map(|s| s.to_string()).collect();
    server.env = env.clone();

    config.mcp_servers.insert(name.to_string(), server);

    let bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Cursor mcp.json at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, &bytes)
}

/// Unregister an MCP server from `~/.cursor/mcp.json`.
/// If `mcpServers` becomes empty and no extra user keys remain, deletes `mcp.json`.
pub fn unregister_cursor_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        std::fs::remove_file(config_path)?;
        return Ok(());
    }

    let mut config: CursorMcpConfig = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Cursor mcp.json at {}: {e}",
            config_path.display()
        ))
    })?;

    config.mcp_servers.remove(name);

    if config.mcp_servers.is_empty() && config.extra.is_empty() {
        std::fs::remove_file(config_path)?;
    } else {
        let bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
            CeError::Runtime(format!(
                "Failed to serialize Cursor mcp.json at {}: {e}",
                config_path.display()
            ))
        })?;
        write_atomic(config_path, &bytes)?;
    }

    Ok(())
}

/// Write or update `.cursor/rules/compound-engineering.mdc` with frontmatter and demarcated managed block.
pub fn update_cursor_rule_mdc(
    rule_path: &Path,
    frontmatter: &CursorRuleFrontmatter,
    managed_text: &str,
) -> Result<(), CeError> {
    let existing_content = if rule_path.exists() {
        std::fs::read_to_string(rule_path)?
    } else {
        String::new()
    };

    let frontmatter_str = format!(
        "---\ndescription: {}\nglobs: {}\nalwaysApply: {}\n---",
        frontmatter.description, frontmatter.globs, frontmatter.always_apply
    );

    let has_frontmatter = existing_content.trim_start().starts_with("---\n")
        || existing_content.trim_start().starts_with("---\r\n");

    let updated_body = update_managed_block(&existing_content, managed_text);

    let final_content = if has_frontmatter {
        updated_body
    } else {
        format!("{}\n\n{}", frontmatter_str, updated_body.trim_start())
    };

    write_atomic(rule_path, final_content.as_bytes())
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

pub const CURSOR_RESUME_COMMAND: &str = "ce-ai workflow resume --json";

const CURSOR_HOOK_EVENTS: [&str; 2] = ["sessionStart", "stop"];

fn has_cursor_event_hook(hooks_obj: &serde_json::Value, event_name: &str) -> bool {
    hooks_obj
        .get(event_name)
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter().any(|entry| {
                entry.get("command").and_then(|c| c.as_str()) == Some(CURSOR_RESUME_COMMAND)
            })
        })
        .unwrap_or(false)
}

/// Checks if `.cursor/hooks.json` contains sessionStart and stop hooks executing `ce-ai workflow resume --json`.
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
    let Some(hooks) = val.get("hooks") else {
        return false;
    };
    CURSOR_HOOK_EVENTS
        .iter()
        .all(|ev| has_cursor_event_hook(hooks, ev))
}

fn ensure_cursor_event_hook(
    hooks_obj: &mut serde_json::Map<String, serde_json::Value>,
    event_name: &str,
) -> Result<bool, CeError> {
    let event_val = hooks_obj
        .entry(event_name)
        .or_insert_with(|| serde_json::json!([]));
    if !event_val.is_array() {
        *event_val = serde_json::json!([]);
    }

    let event_arr = event_val
        .as_array_mut()
        .ok_or_else(|| CeError::Runtime(format!("{event_name} is not an array")))?;

    let target_hook = serde_json::json!({
        "command": CURSOR_RESUME_COMMAND,
    });

    if !event_arr
        .iter()
        .any(|entry| entry.get("command").and_then(|c| c.as_str()) == Some(CURSOR_RESUME_COMMAND))
    {
        event_arr.push(target_hook);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Ensures `.cursor/hooks.json` contains sessionStart and stop hooks for `ce-ai workflow resume --json`.
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

    let mut changed = false;
    for ev in CURSOR_HOOK_EVENTS {
        if ensure_cursor_event_hook(hooks_obj, ev)? {
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    if let Some(parent) = hooks_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| CeError::Runtime(format!("failed to serialize hooks.json: {e}")))?;
    write_atomic(hooks_path, serialized.as_bytes())?;
    Ok(true)
}

/// Surgically removes `ce-ai workflow resume --json` hooks (sessionStart, stop) from `.cursor/hooks.json`.
/// If the file becomes effectively empty or only contains an empty hooks object, cleans up the file.
pub fn remove_session_start_hook(hooks_path: &Path) -> Result<bool, CeError> {
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

    if let Some(hooks_obj) = root_obj.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for ev in CURSOR_HOOK_EVENTS {
            if let Some(event_arr) = hooks_obj.get_mut(ev).and_then(|s| s.as_array_mut()) {
                let orig_len = event_arr.len();
                event_arr.retain(|entry| {
                    entry.get("command").and_then(|c| c.as_str()) != Some(CURSOR_RESUME_COMMAND)
                });
                if event_arr.len() != orig_len {
                    changed = true;
                }
                if event_arr.is_empty() {
                    hooks_obj.remove(ev);
                    changed = true;
                }
            }
        }
        if hooks_obj.is_empty() {
            root_obj.remove("hooks");
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    let only_version = root_obj.len() == 1 && root_obj.contains_key("version");
    if root_obj.is_empty() || only_version {
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
#[path = "tests/cursor.rs"]
mod tests;
