//! opencode.json read → merge (dedup) → atomic write; hard-fails on invalid
//! existing JSON instead of clobbering user config (OI-2, D4).

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::state::write_atomic;

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

/// Returns the mutation record for the install manifest (OI-5).
pub fn ensure_plugin_and_skills(
    config_path: &Path,
    plugin_entry: &str,
    skills_path: &str,
) -> Result<ConfigMutation, CeError> {
    let mut config = read_config(config_path)?;
    merge_plugin(&mut config, plugin_entry)?;
    merge_skills_path(&mut config, skills_path)?;
    write_atomic(config_path, &serde_json::to_vec_pretty(&config)?)?;
    Ok(ConfigMutation {
        file: config_path.display().to_string(),
        backup: None,
        keys: vec!["plugin".into(), "skills.paths".into()],
    })
}

/// Merges an MCP server definition into `opencode.json` under `mcpServers.<tool_name>`.
/// Preserves pre-existing user MCP servers and custom config. Writes atomically.
pub fn register_mcp_server(
    config_path: &Path,
    tool_name: &str,
    server_def: serde_json::Value,
) -> Result<(), CeError> {
    let mut config = read_config(config_path)?;
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
    write_atomic(config_path, &serde_json::to_vec_pretty(&config)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::*;

    fn write_json(path: &Path, value: serde_json::Value) {
        std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }

    fn read_json(path: &Path) -> serde_json::Value {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    fn loader_entry(config_dir: &Path) -> String {
        config_dir
            .join("compound-engineering")
            .join("plugins")
            .join("compound-engineering.js")
            .to_string_lossy()
            .into_owned()
    }

    fn skills_path(config_dir: &Path) -> String {
        config_dir
            .join("compound-engineering")
            .join("skills")
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn merges_plugin_entry_without_clobbering_user_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        write_json(
            &path,
            serde_json::json!({
                "plugin": ["user-plugin"],
                "agent": { "ce-brainstorm": { "model": "user-model" } }
            }),
        );
        let entry = loader_entry(dir.path());
        ensure_plugin_and_skills(&path, &entry, &skills_path(dir.path())).unwrap();

        let config = read_json(&path);
        let plugins = config["plugin"].as_array().expect("plugin is an array");
        assert_eq!(plugins.len(), 2, "user entry plus CE entry");
        assert!(plugins.iter().any(|v| v.as_str() == Some("user-plugin")));
        assert!(plugins.iter().any(|v| v.as_str() == Some(&entry)));
        assert_eq!(config["agent"]["ce-brainstorm"]["model"], "user-model");
    }

    #[test]
    fn reinstall_does_not_duplicate_entries() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        let entry = loader_entry(dir.path());
        let skills = skills_path(dir.path());
        ensure_plugin_and_skills(&path, &entry, &skills).unwrap();
        ensure_plugin_and_skills(&path, &entry, &skills).unwrap();

        let config = read_json(&path);
        assert_eq!(config["plugin"].as_array().unwrap().len(), 1);
        assert_eq!(config["skills"]["paths"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn merges_skills_paths_with_dedup_keeping_user_paths() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        write_json(
            &path,
            serde_json::json!({ "skills": { "paths": ["/home/user/custom-skills"] } }),
        );
        let skills = skills_path(dir.path());
        ensure_plugin_and_skills(&path, &loader_entry(dir.path()), &skills).unwrap();

        let config = read_json(&path);
        let paths = config["skills"]["paths"].as_array().unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths
            .iter()
            .any(|v| v.as_str() == Some("/home/user/custom-skills")));
        assert!(paths.iter().any(|v| v.as_str() == Some(&skills)));
    }

    #[test]
    fn creates_plugin_and_skills_arrays_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        let entry = loader_entry(dir.path());
        let skills = skills_path(dir.path());
        ensure_plugin_and_skills(&path, &entry, &skills).unwrap();

        let config = read_json(&path);
        assert_eq!(config["plugin"], serde_json::json!([entry]));
        assert_eq!(config["skills"]["paths"], serde_json::json!([skills]));
    }

    #[test]
    fn invalid_existing_json_hard_fails_with_fix_guidance() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        std::fs::write(&path, "{ this is not json").unwrap();

        let err = ensure_plugin_and_skills(&path, "plugin-entry", "skills-path").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not valid JSON"), "names the problem: {msg}");
        assert!(msg.contains("opencode.json"), "names the file: {msg}");
        assert!(msg.contains("Fix the file"), "gives fix guidance: {msg}");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{ this is not json",
            "broken config is never overwritten (D4)"
        );
    }

    #[test]
    fn non_array_plugin_key_hard_fails_instead_of_clobbering() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        write_json(&path, serde_json::json!({ "plugin": "not-an-array" }));

        let err = ensure_plugin_and_skills(&path, "plugin-entry", "skills-path").unwrap_err();
        assert!(err.to_string().contains("plugin"));
        assert_eq!(
            read_json(&path)["plugin"],
            "not-an-array",
            "user config preserved"
        );
    }

    #[test]
    fn register_mcp_server_creates_block_and_preserves_malformed_failures() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");

        // 1. Create on empty config
        write_json(&path, serde_json::json!({}));
        register_mcp_server(&path, "context7", serde_json::json!({"command": "npx"})).unwrap();
        let val = read_json(&path);
        assert_eq!(val["mcpServers"]["context7"]["command"], "npx");

        // 2. Fail safely on non-object mcpServers
        write_json(&path, serde_json::json!({ "mcpServers": "invalid-string" }));
        let err =
            register_mcp_server(&path, "rtk", serde_json::json!({"command": "rtk"})).unwrap_err();
        assert!(err.to_string().contains("mcpServers"));
        assert_eq!(read_json(&path)["mcpServers"], "invalid-string");
    }
}
