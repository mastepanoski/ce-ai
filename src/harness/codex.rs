//! Codex AI harness adapter implementation.
//! Handles OpenAI Codex CLI's native `~/.codex/config.toml` (`[mcp_servers.<name>]` TOML schema)
//! and `AGENTS.md` / `.codex/AGENTS.md` instruction files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

        if let Ok(config_env) = std::env::var("CODEX_CONFIG_DIR") {
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

/// Native Codex MCP server entry (`[mcp_servers.<name>]` in TOML).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexMcpServer {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
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
        let env_entry = server_table
            .entry("env".to_string())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if let Some(env_table) = env_entry.as_table_mut() {
            for (k, v) in env {
                env_table.insert(k.clone(), toml::Value::String(v.clone()));
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn codex_adapter_default_paths() {
        let adapter = CodexAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Codex);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.codex/config.toml")
        );
    }

    #[test]
    fn registers_and_unregisters_native_codex_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let mut env = BTreeMap::new();
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        register_codex_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let root: toml::Table = content.parse().unwrap();
        assert!(!root.contains_key("plugin"));
        assert!(!root.contains_key("skills"));

        let mcp = root["mcp_servers"].as_table().unwrap();
        let server: CodexMcpServer = mcp["codegraph"].clone().try_into().unwrap();
        assert_eq!(server.command, "codegraph");
        assert_eq!(server.args, vec!["mcp"]);
        assert_eq!(server.env.get("LOG_LEVEL").unwrap(), "info");

        unregister_codex_mcp_server(&config_path, "codegraph").unwrap();
        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let root_after: toml::Table = content_after.parse().unwrap();
        if let Some(mcp_after) = root_after.get("mcp_servers").and_then(|v| v.as_table()) {
            assert!(!mcp_after.contains_key("codegraph"));
        }
    }

    #[test]
    fn preserves_existing_user_codex_tables_and_extra_fields() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let initial_toml = r#"model = "gpt-4o"

[mcp_servers.engram]
command = "engram"
args = ["serve"]
enabled = true
"#;
        std::fs::write(&config_path, initial_toml).unwrap();

        let env = BTreeMap::new();
        register_codex_mcp_server(
            &config_path,
            "engram",
            "engram",
            &["serve", "--debug"],
            &env,
        )
        .unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let root: toml::Table = content.parse().unwrap();
        assert_eq!(root["model"].as_str().unwrap(), "gpt-4o");
        let mcp = root["mcp_servers"].as_table().unwrap();
        let engram_table = mcp["engram"].as_table().unwrap();
        assert!(engram_table["enabled"].as_bool().unwrap());
        assert_eq!(engram_table["command"].as_str().unwrap(), "engram");

        unregister_codex_mcp_server(&config_path, "engram").unwrap();

        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let root_after: toml::Table = content_after.parse().unwrap();
        assert_eq!(root_after["model"].as_str().unwrap(), "gpt-4o");
    }

    #[test]
    fn updates_and_strips_codex_agents_md_managed_block() {
        let tmp = TempDir::new().unwrap();
        let md_path = tmp.path().join("AGENTS.md");

        let user_header = "# My Project\n";
        std::fs::write(&md_path, user_header).unwrap();

        update_codex_agents_md(&md_path, "Directives content").unwrap();

        let content = std::fs::read_to_string(&md_path).unwrap();
        assert!(content.starts_with("# My Project"));
        assert!(content.contains(CE_MANAGED_BEGIN));
        assert!(content.contains("Directives content"));
        assert!(content.contains(CE_MANAGED_END));

        let stripped = strip_managed_block(&content);
        assert!(!stripped.contains(CE_MANAGED_BEGIN));
        assert_eq!(stripped.trim(), "# My Project");
    }
}
