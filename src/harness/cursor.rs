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

#[cfg(test)]
#[path = "tests/cursor.rs"]
mod tests;
