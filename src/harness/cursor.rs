//! Cursor native harness adapter implementation.
//! Handles Cursor's native `~/.cursor/mcp.json` (`mcpServers` stdio schema)
//! and `.cursor/rules/*.mdc` instruction files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct CursorAdapter;

impl HarnessAdapter for CursorAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Cursor
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        home.join(".cursor").join("mcp.json")
    }
}

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

/// Strip demarcated managed comment block on uninstallation.
#[allow(dead_code)]
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
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cursor_adapter_default_paths() {
        let adapter = CursorAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Cursor);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.cursor/mcp.json")
        );
    }

    #[test]
    fn registers_and_unregisters_native_cursor_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let mut env = BTreeMap::new();
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        register_cursor_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"mcpServers\""));
        assert!(content.contains("\"codegraph\""));
        assert!(content.contains("\"stdio\""));
        assert!(!content.contains("\"plugin\""));
        assert!(!content.contains("\"skills\""));

        let config: CursorMcpConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);
        assert_eq!(
            config.mcp_servers["codegraph"].r#type.as_deref(),
            Some("stdio")
        );

        unregister_cursor_mcp_server(&config_path, "codegraph").unwrap();
        assert!(!config_path.exists());
    }

    #[test]
    fn preserves_existing_user_cursor_mcp_servers_and_per_server_extra_fields() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let initial_json = r#"{
  "mcpServers": {
    "user-tool": {
      "type": "stdio",
      "command": "node",
      "args": ["server.js"],
      "disabled": false,
      "timeout": 300
    },
    "sse-tool": {
      "type": "sse",
      "url": "https://mcp.example.com/sse"
    }
  },
  "userSetting": true
}"#;
        std::fs::write(&config_path, initial_json).unwrap();

        let env = BTreeMap::new();
        register_cursor_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: CursorMcpConfig = serde_json::from_str(&content).unwrap();

        assert_eq!(config.mcp_servers.len(), 3);
        assert!(config.mcp_servers.contains_key("user-tool"));
        assert!(config.mcp_servers.contains_key("sse-tool"));
        assert!(config.mcp_servers.contains_key("engram"));
        assert_eq!(
            config.mcp_servers["user-tool"].extra.get("disabled"),
            Some(&serde_json::Value::Bool(false))
        );
        assert_eq!(
            config.mcp_servers["sse-tool"].url,
            Some("https://mcp.example.com/sse".to_string())
        );

        unregister_cursor_mcp_server(&config_path, "engram").unwrap();

        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: CursorMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert_eq!(config_after.mcp_servers.len(), 2);
        assert!(config_after.mcp_servers.contains_key("user-tool"));
        assert!(config_after.mcp_servers.contains_key("sse-tool"));
    }

    #[test]
    fn updates_cursor_rule_mdc_with_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let rule_path = tmp.path().join("compound-engineering.mdc");
        let frontmatter = CursorRuleFrontmatter::default();

        update_cursor_rule_mdc(&rule_path, &frontmatter, "Directives content").unwrap();

        let content = std::fs::read_to_string(&rule_path).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("description: Compound Engineering Agent Directives"));
        assert!(content.contains("globs: *"));
        assert!(content.contains("alwaysApply: true"));
        assert!(content.contains(CE_MANAGED_BEGIN));
        assert!(content.contains("Directives content"));
        assert!(content.contains(CE_MANAGED_END));
    }

    #[test]
    fn handles_unbalanced_managed_markers_gracefully() {
        let unbalanced_start = format!("User content\n\n{}\nPartial block", CE_MANAGED_BEGIN);
        let updated = update_managed_block(&unbalanced_start, "Fresh block");
        assert!(updated.contains("User content"));
        assert!(updated.contains(CE_MANAGED_BEGIN));
        assert!(updated.contains("Fresh block"));
        assert!(updated.contains(CE_MANAGED_END));

        let stripped = strip_managed_block(&unbalanced_start);
        assert_eq!(stripped.trim(), "User content");
    }

    #[test]
    fn preserves_non_stdio_transport_type_on_re_registration() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");
        let initial_json = r#"{
  "mcpServers": {
    "engram": {
      "type": "sse",
      "command": "old",
      "url": "https://example.com/sse"
    }
  }
}"#;
        std::fs::write(&config_path, initial_json).unwrap();
        let env = BTreeMap::new();
        register_cursor_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: CursorMcpConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.mcp_servers["engram"].r#type.as_deref(), Some("sse"));
        assert_eq!(config.mcp_servers["engram"].command, "engram");
    }

    #[test]
    fn updates_existing_cursor_rule_mdc_without_duplicating_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let rule_path = tmp.path().join("compound-engineering.mdc");
        let frontmatter = CursorRuleFrontmatter::default();

        let initial_text = r#"---
description: Custom Rule
globs: *.rs
alwaysApply: false
---
# User Custom Instructions
Keep types strict.
"#;
        std::fs::write(&rule_path, initial_text).unwrap();

        update_cursor_rule_mdc(&rule_path, &frontmatter, "Updated managed text").unwrap();

        let content = std::fs::read_to_string(&rule_path).unwrap();
        let frontmatter_count = content.matches("---").count();
        assert_eq!(
            frontmatter_count, 2,
            "Frontmatter delimiters must not be duplicated"
        );
        assert!(content.contains("# User Custom Instructions"));
        assert!(content.contains("Keep types strict."));
        assert!(content.contains("Updated managed text"));
    }
}
