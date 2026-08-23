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
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn grok_adapter_default_paths() {
        let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("GROK_HOME");
        let adapter = GrokAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Grok);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.grok/config.toml")
        );
    }

    #[test]
    fn grok_adapter_respects_grok_home_env() {
        let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
        let adapter = GrokAdapter;
        let home = PathBuf::from("/tmp/home");
        std::env::set_var("GROK_HOME", "/custom/grok/dir");
        let path = adapter.default_config_path(&home);
        std::env::remove_var("GROK_HOME");
        assert_eq!(path, PathBuf::from("/custom/grok/dir/config.toml"));
    }

    #[test]
    fn grok_adapter_config_path_edge_cases() {
        let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("GROK_HOME");
        let adapter = GrokAdapter;

        let config_direct = PathBuf::from("/tmp/home/config.toml");
        assert_eq!(adapter.default_config_path(&config_direct), config_direct);

        let grok_dir_direct = PathBuf::from("/tmp/home/.grok");
        assert_eq!(
            adapter.default_config_path(&grok_dir_direct),
            PathBuf::from("/tmp/home/.grok/config.toml")
        );
    }

    #[test]
    fn register_grok_mcp_server_handles_invalid_toml() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, "invalid_toml = [unclosed").unwrap();

        let env = BTreeMap::new();
        let res = register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &env);
        assert!(res.is_err());
    }

    #[test]
    fn registers_and_unregisters_native_grok_mcp_server() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let mut env = BTreeMap::new();
        env.insert("LOG_LEVEL".to_string(), "info".to_string());

        register_grok_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let root: toml::Table = content.parse().unwrap();
        let mcp = root["mcp_servers"].as_table().unwrap();
        assert!(mcp.contains_key("codegraph"));

        let codegraph = mcp["codegraph"].as_table().unwrap();
        assert_eq!(codegraph["command"].as_str().unwrap(), "codegraph");
        assert_eq!(
            codegraph["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["mcp"]
        );
        assert!(!root.contains_key("plugin"), "Zero OpenCode key leaks");
        assert!(!root.contains_key("skills"), "Zero OpenCode key leaks");

        unregister_grok_mcp_server(&config_path, "codegraph").unwrap();
        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let root_after: toml::Table = content_after.parse().unwrap();
        let mcp_after = root_after["mcp_servers"].as_table().unwrap();
        assert!(!mcp_after.contains_key("codegraph"));
    }

    #[test]
    fn preserves_existing_user_grok_tables_and_extra_fields() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let initial_toml = r#"
model = "grok-beta"

[mcp_servers.engram]
command = "engram"
args = ["serve"]
enabled = true
"#;
        std::fs::write(&config_path, initial_toml).unwrap();

        let env = BTreeMap::new();
        register_grok_mcp_server(
            &config_path,
            "engram",
            "engram",
            &["serve", "--debug"],
            &env,
        )
        .unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let root: toml::Table = content.parse().unwrap();
        assert_eq!(root["model"].as_str().unwrap(), "grok-beta");
        let mcp = root["mcp_servers"].as_table().unwrap();
        let engram_table = mcp["engram"].as_table().unwrap();
        assert!(engram_table["enabled"].as_bool().unwrap());
        assert_eq!(engram_table["command"].as_str().unwrap(), "engram");

        unregister_grok_mcp_server(&config_path, "engram").unwrap();

        let content_after = std::fs::read_to_string(&config_path).unwrap();
        let root_after: toml::Table = content_after.parse().unwrap();
        assert_eq!(root_after["model"].as_str().unwrap(), "grok-beta");
    }

    #[test]
    fn updates_and_strips_grok_rule_md_managed_block() {
        let tmp = TempDir::new().unwrap();
        let md_path = tmp.path().join("compound-engineering.md");

        let user_header = "# My Grok Rules\n";
        std::fs::write(&md_path, user_header).unwrap();

        update_grok_rule_md(&md_path, "Directives content").unwrap();

        let content = std::fs::read_to_string(&md_path).unwrap();
        assert!(content.starts_with("# My Grok Rules"));
        assert!(content.contains(CE_MANAGED_BEGIN));
        assert!(content.contains("Directives content"));
        assert!(content.contains(CE_MANAGED_END));

        let stripped = strip_managed_block(&content);
        assert!(!stripped.contains(CE_MANAGED_BEGIN));
        assert_eq!(stripped.trim(), "# My Grok Rules");
    }

    #[test]
    fn replaces_env_map_cleanly_on_re_registration() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        let mut env1 = BTreeMap::new();
        env1.insert("OLD_KEY".to_string(), "old_val".to_string());
        register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &env1).unwrap();

        let mut env2 = BTreeMap::new();
        env2.insert("NEW_KEY".to_string(), "new_val".to_string());
        register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &env2).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        let root: toml::Table = content.parse().unwrap();
        let engram_env = root["mcp_servers"]["engram"]["env"].as_table().unwrap();
        assert!(!engram_env.contains_key("OLD_KEY"));
        assert_eq!(engram_env["NEW_KEY"].as_str().unwrap(), "new_val");

        // Re-register with empty env map -> removes env table
        let empty_env = BTreeMap::new();
        register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &empty_env).unwrap();
        let content_empty = std::fs::read_to_string(&config_path).unwrap();
        let root_empty: toml::Table = content_empty.parse().unwrap();
        let engram_table = root_empty["mcp_servers"]["engram"].as_table().unwrap();
        assert!(!engram_table.contains_key("env"));
    }
}
