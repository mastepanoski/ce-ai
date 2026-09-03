//! Codex AI harness adapter implementation.
//! Handles OpenAI Codex CLI's native `~/.codex/config.toml` (`[mcp_servers.<name>]` TOML schema)
//! and `AGENTS.md` / `.codex/AGENTS.md` instruction files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct CodexAdapter;

impl HarnessAdapter for CodexAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Codex
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("config.toml") {
            return home.to_path_buf();
        }

        if let Some(config_env) = std::env::var_os("CODEX_HOME") {
            return PathBuf::from(config_env).join("config.toml");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".codex") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".codex").join("config.toml")
    }
}

/// Merge and register an MCP server into Codex's TOML config using native `[mcp_servers.<name>]` schema.
pub fn register_codex_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut root_table: toml::Table = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            toml::Table::new()
        } else {
            content.parse::<toml::Table>().map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Codex config.toml at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        toml::Table::new()
    };

    let mcp_servers_entry = root_table
        .entry("mcp_servers".to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    let mcp_servers_table = mcp_servers_entry.as_table_mut().ok_or_else(|| {
        CeError::Runtime(format!(
            "Key `mcp_servers` in Codex config at {} is not a table",
            config_path.display()
        ))
    })?;

    let server_table = mcp_servers_table
        .entry(name.to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            CeError::Runtime(format!(
                "Key `mcp_servers.{name}` in Codex config at {} is not a table",
                config_path.display()
            ))
        })?;

    server_table.insert(
        "command".to_string(),
        toml::Value::String(command.to_string()),
    );
    let args_vec = args
        .iter()
        .map(|s| toml::Value::String(s.to_string()))
        .collect();
    server_table.insert("args".to_string(), toml::Value::Array(args_vec));

    if !env.is_empty() {
        let mut env_table = toml::Table::new();
        for (k, v) in env {
            env_table.insert(k.clone(), toml::Value::String(v.clone()));
        }
        server_table.insert("env".to_string(), toml::Value::Table(env_table));
    } else {
        server_table.remove("env");
    }

    let toml_string = toml::to_string_pretty(&root_table).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Codex config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, toml_string.as_bytes())
}

/// Unregister an MCP server from Codex's TOML configuration file.
/// Removes the specified server table from `mcp_servers`. Leaves file intact to preserve user preferences.
pub fn unregister_codex_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        return Ok(());
    }

    let mut root_table: toml::Table = content.parse::<toml::Table>().map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Codex config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    if let Some(mcp_servers_entry) = root_table.get_mut("mcp_servers") {
        if let Some(mcp_servers_table) = mcp_servers_entry.as_table_mut() {
            mcp_servers_table.remove(name);
        }
    }

    let toml_string = toml::to_string_pretty(&root_table).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Codex config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, toml_string.as_bytes())
}

/// Write or update project directives in `./AGENTS.md` or `.codex/AGENTS.md` with demarcated managed block.
pub fn update_codex_agents_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError> {
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

pub const CODEX_RESUME_COMMAND: &str = "ce-ai workflow resume";

/// Checks if `.codex/config.toml` contains a SessionStart hook executing `ce-ai workflow resume`.
pub fn has_session_start_hook(config_path: &Path) -> bool {
    if !config_path.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(config_path) else {
        return false;
    };
    let Ok(root_table) = content.parse::<toml::Table>() else {
        return false;
    };
    root_table
        .get("hooks")
        .and_then(|h| h.as_table())
        .and_then(|t| t.get("SessionStart"))
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter().any(|entry| {
                entry
                    .as_table()
                    .and_then(|e| e.get("hooks"))
                    .and_then(|h| h.as_array())
                    .map(|h_arr| {
                        h_arr.iter().any(|cmd| {
                            cmd.as_table()
                                .and_then(|c| c.get("command"))
                                .and_then(|s| s.as_str())
                                == Some(CODEX_RESUME_COMMAND)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Ensures `.codex/config.toml` contains the SessionStart hook for `ce-ai workflow resume`.
/// Preserves any pre-existing user hooks or extra settings. Idempotent.
pub fn ensure_session_start_hook(config_path: &Path) -> Result<bool, CeError> {
    if has_session_start_hook(config_path) {
        return Ok(false);
    }

    let mut root_table: toml::Table = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            toml::Table::new()
        } else {
            content.parse::<toml::Table>().map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Codex config.toml at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        toml::Table::new()
    };

    let hooks_val = root_table
        .entry("hooks".to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    if !hooks_val.is_table() {
        *hooks_val = toml::Value::Table(toml::Table::new());
    }
    let hooks_table = hooks_val.as_table_mut().ok_or_else(|| {
        CeError::Runtime(format!(
            "Key `hooks` in Codex config at {} is not a table",
            config_path.display()
        ))
    })?;

    let session_start_val = hooks_table
        .entry("SessionStart".to_string())
        .or_insert_with(|| toml::Value::Array(Vec::new()));
    if !session_start_val.is_array() {
        *session_start_val = toml::Value::Array(Vec::new());
    }
    let session_start_arr = session_start_val.as_array_mut().ok_or_else(|| {
        CeError::Runtime(format!(
            "Key `hooks.SessionStart` in Codex config at {} is not an array",
            config_path.display()
        ))
    })?;

    let mut target_hook = toml::Table::new();
    target_hook.insert(
        "type".to_string(),
        toml::Value::String("command".to_string()),
    );
    target_hook.insert(
        "command".to_string(),
        toml::Value::String(CODEX_RESUME_COMMAND.to_string()),
    );
    target_hook.insert(
        "statusMessage".to_string(),
        toml::Value::String("Loading ce-ai workflow state".to_string()),
    );

    let existing_entry = session_start_arr.iter_mut().find(|entry| {
        if let Some(t) = entry.as_table() {
            t.get("matcher")
                .and_then(|m| m.as_str())
                .map(|m| {
                    m.contains("startup")
                        || m.contains("resume")
                        || m.contains("compact")
                        || m == ".*"
                })
                .unwrap_or(false)
                && t.get("hooks").and_then(|h| h.as_array()).is_some()
        } else {
            false
        }
    });

    match existing_entry {
        Some(entry) => {
            if let Some(hooks_arr) = entry
                .as_table_mut()
                .and_then(|t| t.get_mut("hooks"))
                .and_then(|h| h.as_array_mut())
            {
                if !hooks_arr.iter().any(|h| {
                    h.as_table()
                        .and_then(|t| t.get("command"))
                        .and_then(|c| c.as_str())
                        == Some(CODEX_RESUME_COMMAND)
                }) {
                    hooks_arr.push(toml::Value::Table(target_hook));
                }
            }
        }
        None => {
            let mut entry = toml::Table::new();
            entry.insert(
                "matcher".to_string(),
                toml::Value::String("startup|resume|compact".to_string()),
            );
            entry.insert(
                "hooks".to_string(),
                toml::Value::Array(vec![toml::Value::Table(target_hook)]),
            );
            session_start_arr.push(toml::Value::Table(entry));
        }
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let toml_string = toml::to_string_pretty(&root_table).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Codex config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, toml_string.as_bytes())?;
    Ok(true)
}

/// Surgically removes `ce-ai workflow resume` hook from `.codex/config.toml`.
/// If the file becomes empty `{}` as a result, removes the file cleanly.
pub fn remove_session_start_hook(config_path: &Path) -> Result<bool, CeError> {
    if !config_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        return Ok(false);
    }

    let Ok(mut root_table) = content.parse::<toml::Table>() else {
        return Ok(false);
    };

    let mut changed = false;

    if let Some(hooks_table) = root_table.get_mut("hooks").and_then(|h| h.as_table_mut()) {
        if let Some(session_start_arr) = hooks_table
            .get_mut("SessionStart")
            .and_then(|s| s.as_array_mut())
        {
            for entry in session_start_arr.iter_mut() {
                if let Some(hooks_arr) = entry
                    .as_table_mut()
                    .and_then(|t| t.get_mut("hooks"))
                    .and_then(|h| h.as_array_mut())
                {
                    let prev_len = hooks_arr.len();
                    hooks_arr.retain(|h| {
                        h.as_table()
                            .and_then(|t| t.get("command"))
                            .and_then(|c| c.as_str())
                            != Some(CODEX_RESUME_COMMAND)
                    });
                    if hooks_arr.len() != prev_len {
                        changed = true;
                    }
                }
            }

            let prev_len = session_start_arr.len();
            session_start_arr.retain(|entry| {
                entry
                    .as_table()
                    .and_then(|t| t.get("hooks"))
                    .and_then(|h| h.as_array())
                    .map(|h| !h.is_empty())
                    .unwrap_or(false)
            });
            if session_start_arr.len() != prev_len {
                changed = true;
            }

            if session_start_arr.is_empty() {
                hooks_table.remove("SessionStart");
                changed = true;
            }
        }

        if hooks_table.is_empty() {
            root_table.remove("hooks");
            changed = true;
        }
    }

    if !changed {
        return Ok(false);
    }

    if root_table.is_empty() {
        let _ = std::fs::remove_file(config_path);
        if let Some(parent) = config_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
        return Ok(true);
    }

    let toml_string = toml::to_string_pretty(&root_table).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Codex config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, toml_string.as_bytes())?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/codex.rs"]
mod tests;
