//! xAI Grok Build AI harness adapter implementation.
//! Handles Grok Build CLI's native `~/.grok/config.toml` (`[mcp_servers.<name>]` TOML schema)
//! and `.grok/rules/compound-engineering.md` instruction file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct GrokAdapter;

impl HarnessAdapter for GrokAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Grok
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("config.toml") {
            return home.to_path_buf();
        }

        if let Some(config_env) = std::env::var_os("GROK_HOME") {
            return PathBuf::from(config_env).join("config.toml");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".grok") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".grok").join("config.toml")
    }
}

/// Merge and register an MCP server into Grok's TOML config using native `[mcp_servers.<name>]` schema.
pub fn register_grok_mcp_server(
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
            content.parse().map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Grok config.toml at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        toml::Table::new()
    };

    let mcp_servers = root_table
        .entry("mcp_servers".to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    let mcp_table = mcp_servers.as_table_mut().ok_or_else(|| {
        CeError::Runtime(format!(
            "mcp_servers in Grok config.toml at {} is not a table",
            config_path.display()
        ))
    })?;

    let server_entry = mcp_table
        .entry(name.to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));

    let server_table = server_entry.as_table_mut().ok_or_else(|| {
        CeError::Runtime(format!(
            "mcp_servers.{} in Grok config.toml at {} is not a table",
            name,
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
            "Failed to serialize Grok config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, toml_string.as_bytes())
}

/// Unregister an MCP server from Grok's TOML configuration file.
/// Removes the specified server entry from `[mcp_servers]`. Leaves file intact to preserve user preferences.
pub fn unregister_grok_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        return Ok(());
    }

    let mut root_table: toml::Table = content.parse().map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Grok config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    if let Some(mcp_servers) = root_table
        .get_mut("mcp_servers")
        .and_then(|v| v.as_table_mut())
    {
        mcp_servers.remove(name);
    }

    let toml_string = toml::to_string_pretty(&root_table).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize Grok config.toml at {}: {e}",
            config_path.display()
        ))
    })?;

    write_atomic(config_path, toml_string.as_bytes())
}

/// Write or update project directives in `.grok/rules/compound-engineering.md` with demarcated managed block.
pub fn update_grok_rule_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError> {
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
#[path = "tests/grok.rs"]
mod tests;
