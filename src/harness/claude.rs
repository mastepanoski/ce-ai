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
            let json_path = env_path.join("claude.json");
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn claude_adapter_default_paths() {
        let adapter = ClaudeAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Claude);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.claude.json")
        );
    }

    #[test]
    fn registers_and_unregisters_native_claude_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join(".claude.json");

        let mut env = BTreeMap::new();
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        register_claude_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("\"mcpServers\""));
        assert!(content.contains("\"codegraph\""));
        assert!(!content.contains("\"plugin\""));
        assert!(!content.contains("\"skills\""));

        let config: ClaudeMcpConfig = serde_json::from_str(&content).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);

        unregister_claude_mcp_server(&config_path, "codegraph").unwrap();
        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: ClaudeMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert!(config_after.mcp_servers.is_empty());
    }

    #[test]
    fn preserves_existing_user_claude_mcp_servers_and_extra_fields() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join(".claude.json");

        let initial_json = r#"{
  "mcpServers": {
    "user-tool": {
      "command": "node",
      "args": ["server.js"],
      "disabled": false
    }
  },
  "numStartups": 42
}"#;
        std::fs::write(&config_path, initial_json).unwrap();

        let env = BTreeMap::new();
        register_claude_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let config: ClaudeMcpConfig = serde_json::from_str(&content).unwrap();

        assert_eq!(config.mcp_servers.len(), 2);
        assert!(config.mcp_servers.contains_key("user-tool"));
        assert!(config.mcp_servers.contains_key("engram"));
        assert_eq!(config.mcp_servers["user-tool"].r#type, None);
        assert_eq!(
            config.extra.get("numStartups"),
            Some(&serde_json::Value::Number(42.into()))
        );

        unregister_claude_mcp_server(&config_path, "engram").unwrap();

        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let config_after: ClaudeMcpConfig = serde_json::from_str(&content_after).unwrap();
        assert_eq!(config_after.mcp_servers.len(), 1);
        assert!(config_after.mcp_servers.contains_key("user-tool"));
    }

    #[test]
    fn updates_and_strips_claude_md_managed_block() {
        let tmp = TempDir::new().unwrap();
        let md_path = tmp.path().join("CLAUDE.md");

        update_claude_md(&md_path, "Directives content").unwrap();

        let content = std::fs::read_to_string(&md_path).unwrap();
        assert!(content.contains(CE_MANAGED_BEGIN));
        assert!(content.contains("Directives content"));
        assert!(content.contains(CE_MANAGED_END));

        let stripped = strip_managed_block(&content);
        assert!(!stripped.contains(CE_MANAGED_BEGIN));
    }
}
