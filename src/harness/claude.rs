//! Claude Code native harness adapter implementation.
//! Handles Claude Code's native `~/.claude.json` / `~/.claude/settings.json` (`mcpServers` stdio schema)
//! and `CLAUDE.md` / `.claude/CLAUDE.md` instruction files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct ClaudeAdapter;

impl HarnessAdapter for ClaudeAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Claude
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".claude") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        if let Ok(config_env) = std::env::var("CLAUDE_CONFIG_DIR") {
            let env_path = PathBuf::from(config_env);
            let settings = env_path.join("settings.json");
            if settings.exists() {
                return settings;
            }
            let json_path = env_path.join(".claude.json");
            if json_path.exists() {
                return json_path;
            }
            return env_path.join(".claude.json");
        }

        let settings_path = home_dir.join(".claude").join("settings.json");
        if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                if content.contains("\"mcpServers\"") {
                    return settings_path;
                }
            }
        }
        home_dir.join(".claude.json")
    }
}

/// Root schema for Claude Code user configuration (`~/.claude.json` / `~/.claude/settings.json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClaudeMcpConfig {
    #[serde(
        rename = "mcpServers",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub mcp_servers: BTreeMap<String, ClaudeMcpServer>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Native Claude Code MCP server entry (stdio or SSE/http transport).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeMcpServer {
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

/// Merge and register an MCP server into Claude's config using native `mcpServers` schema.
pub fn register_claude_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: ClaudeMcpConfig = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            ClaudeMcpConfig::default()
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Claude config at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        ClaudeMcpConfig::default()
    };

    let mut server = config
        .mcp_servers
        .remove(name)
        .unwrap_or_else(|| ClaudeMcpServer {
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
            "Failed to serialize Claude config at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, &bytes)
}

/// Unregister an MCP server from Claude's configuration file.
/// If no other `mcpServers` or extra settings remain AND file was empty initially, cleans up.
pub fn unregister_claude_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        std::fs::remove_file(config_path)?;
        return Ok(());
    }

    let mut config: ClaudeMcpConfig = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Claude config at {}: {e}",
            config_path.display()
        ))
    })?;

    config.mcp_servers.remove(name);

    let bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Claude config at {}: {e}",
            config_path.display()
        ))
    })?;
    write_atomic(config_path, &bytes)?;

    Ok(())
}

/// Write or update project directives in `./CLAUDE.md` or `.claude/CLAUDE.md` with demarcated managed block.
pub fn update_claude_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError> {
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

pub const RESUME_COMMAND: &str = "ce-ai workflow resume";

const CLAUDE_HOOK_EVENTS: [&str; 3] = ["SessionStart", "Stop", "PreCompact"];

fn has_event_hook(hooks_obj: &serde_json::Value, event_name: &str) -> bool {
    hooks_obj
        .get(event_name)
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(|h_arr| h_arr.as_array())
                    .map(|cmds| {
                        cmds.iter().any(|cmd| {
                            cmd.get("command").and_then(|c| c.as_str()) == Some(RESUME_COMMAND)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Checks if `.claude/settings.json` contains SessionStart, Stop, and PreCompact hooks executing `ce-ai workflow resume`.
pub fn has_session_start_hook(settings_path: &Path) -> bool {
    if !settings_path.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(settings_path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    let Some(hooks) = val.get("hooks") else {
        return false;
    };
    CLAUDE_HOOK_EVENTS
        .iter()
        .all(|ev| has_event_hook(hooks, ev))
}

fn ensure_event_hook(
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
        "type": "command",
        "command": RESUME_COMMAND
    });

    let wildcard_entry = event_arr.iter_mut().find(|entry| {
        entry.get("matcher").and_then(|m| m.as_str()) == Some(".*")
            && entry.get("hooks").and_then(|h| h.as_array()).is_some()
    });

    let mut changed = false;
    match wildcard_entry {
        Some(entry) => {
            if let Some(hooks_list) = entry.get_mut("hooks").and_then(|h| h.as_array_mut()) {
                if !hooks_list
                    .iter()
                    .any(|c| c.get("command").and_then(|s| s.as_str()) == Some(RESUME_COMMAND))
                {
                    hooks_list.push(target_hook);
                    changed = true;
                }
            }
        }
        None => {
            event_arr.push(serde_json::json!({
                "matcher": ".*",
                "hooks": [target_hook]
            }));
            changed = true;
        }
    }
    Ok(changed)
}

/// Ensures `.claude/settings.json` contains SessionStart, Stop, and PreCompact hooks for `ce-ai workflow resume`.
/// Preserves any pre-existing user hooks or extra settings. Idempotent.
pub fn ensure_session_start_hook(settings_path: &Path) -> Result<bool, CeError> {
    if has_session_start_hook(settings_path) {
        return Ok(false);
    }

    let mut root: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(settings_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| CeError::Runtime("settings root is not an object".to_string()))?;
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
    for ev in CLAUDE_HOOK_EVENTS {
        if ensure_event_hook(hooks_obj, ev)? {
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| CeError::Runtime(format!("failed to serialize settings.json: {e}")))?;
    write_atomic(settings_path, serialized.as_bytes())?;
    Ok(true)
}

/// Surgically removes `ce-ai workflow resume` hooks (SessionStart, Stop, PreCompact) from `.claude/settings.json`.
/// If the file becomes empty `{}` as a result, removes the file cleanly.
pub fn remove_session_start_hook(settings_path: &Path) -> Result<bool, CeError> {
    if !settings_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(settings_path)?;
    let Ok(mut root) = serde_json::from_str::<serde_json::Value>(&content) else {
        return Ok(false);
    };

    let mut changed = false;

    if let Some(hooks_obj) = root.get_mut("hooks").and_then(|h| h.as_object_mut()) {
        for ev in CLAUDE_HOOK_EVENTS {
            if let Some(arr) = hooks_obj.get_mut(ev).and_then(|s| s.as_array_mut()) {
                for entry in arr.iter_mut() {
                    if let Some(hooks_list) = entry.get_mut("hooks").and_then(|h| h.as_array_mut())
                    {
                        let prev_len = hooks_list.len();
                        hooks_list.retain(|cmd| {
                            cmd.get("command").and_then(|c| c.as_str()) != Some(RESUME_COMMAND)
                        });
                        if hooks_list.len() != prev_len {
                            changed = true;
                        }
                    }
                }

                let prev_len = arr.len();
                arr.retain(|entry| {
                    entry
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|h| !h.is_empty())
                        .unwrap_or(false)
                });
                if arr.len() != prev_len {
                    changed = true;
                }

                if arr.is_empty() {
                    hooks_obj.remove(ev);
                    changed = true;
                }
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

    if root.as_object().map(|o| o.is_empty()).unwrap_or(false) {
        let _ = std::fs::remove_file(settings_path);
        return Ok(true);
    }

    let serialized = serde_json::to_string_pretty(&root)
        .map_err(|e| CeError::Runtime(format!("failed to serialize settings.json: {e}")))?;
    write_atomic(settings_path, serialized.as_bytes())?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/claude.rs"]
mod tests;
