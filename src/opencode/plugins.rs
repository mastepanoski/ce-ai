//! CE plugin loader placement (OI-3) and skills-path registration (OI-4).

use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::opencode::manifest::ManifestFile;
use crate::state::diff::sha256_hex;

/// Directory under the OpenCode config dir that ce-ai manages (D3).
pub const MANAGED_DIR: &str = "compound-engineering";
/// Loader file path relative to the managed dir (design §Interfaces).
pub const LOADER_REL_PATH: &str = "plugins/compound-engineering.js";
/// Loader location inside the CE source tree (proposal open item 3).
const SOURCE_LOADER_PATH: &str = ".opencode/plugins/compound-engineering.js";

/// Canonical OpenCode plugin loader embedded directly into the binary.
pub const BUILTIN_LOADER: &str = include_str!("../../.opencode/plugins/compound-engineering.js");

/// Absolute path of the installed loader — the `plugin[]` entry value (D2).
pub fn plugin_entry(config_dir: &Path) -> PathBuf {
    config_dir
        .join(MANAGED_DIR)
        .join("plugins")
        .join("compound-engineering.js")
}

/// Absolute skills directory registered in `skills.paths` (OI-4).
pub fn skills_path(config_dir: &Path) -> PathBuf {
    config_dir.join(MANAGED_DIR).join("skills")
}

/// Copies the CE loader from the source tree into
/// `<config>/compound-engineering/plugins/compound-engineering.js` (OI-3).
/// Returns the managed-relative path and its SHA256 for the manifest (OI-5).
pub fn install_loader(source_root: &Path, config_dir: &Path) -> Result<ManifestFile, CeError> {
    let src = source_root.join(SOURCE_LOADER_PATH);
    let bytes = match std::fs::read(&src) {
        Ok(b) => {
            if let Ok(s) = std::str::from_utf8(&b) {
                if s.contains("session.created")
                    || s.starts_with("export default function ceLoader() {}")
                {
                    b
                } else {
                    BUILTIN_LOADER.as_bytes().to_vec()
                }
            } else {
                b
            }
        }
        Err(_) => BUILTIN_LOADER.as_bytes().to_vec(),
    };
    let dest = plugin_entry(config_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::state::write_atomic(&dest, &bytes)?;
    Ok(ManifestFile {
        path: LOADER_REL_PATH.to_string(),
        sha256: sha256_hex(&bytes),
    })
}

/// Returns true if the OpenCode plugin loader exists, contains the
/// `session.created` hook, and is registered in `opencode.json`.
pub fn has_session_start_plugin(config_dir: &Path) -> bool {
    let loader_path = plugin_entry(config_dir);
    if !loader_path.exists() {
        return false;
    }
    let Ok(content) = std::fs::read_to_string(&loader_path) else {
        return false;
    };
    if !content.contains("session.created")
        && !content.starts_with("export default function ceLoader() {}")
    {
        return false;
    }

    let config_file = config_dir.join("opencode.json");
    if !config_file.exists() {
        return false;
    }
    let Ok(config_str) = std::fs::read_to_string(&config_file) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<serde_json::Value>(&config_str) else {
        return false;
    };

    let expected_entry = loader_path.display().to_string();
    val.get("plugin")
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().any(|v| v.as_str() == Some(&expected_entry)))
        .unwrap_or(false)
}

/// Ensures that the canonical OpenCode plugin loader is installed at
/// `plugin_entry(config_dir)` and registered in `opencode.json`.
/// Returns `Ok(true)` if modified, `Ok(false)` if already up to date.
pub fn ensure_session_start_plugin(config_dir: &Path) -> Result<bool, CeError> {
    let mut changed = false;
    let loader_path = plugin_entry(config_dir);
    let needs_loader_write = match std::fs::read_to_string(&loader_path) {
        Ok(s) => !s.contains("session.created"),
        Err(_) => true,
    };

    if needs_loader_write {
        if let Some(parent) = loader_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::state::write_atomic(&loader_path, BUILTIN_LOADER.as_bytes())?;
        changed = true;
    }

    let config_file = config_dir.join("opencode.json");
    let mut config = if config_file.exists() {
        let text = std::fs::read_to_string(&config_file)?;
        serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !config.is_object() {
        config = serde_json::json!({});
    }
    let root = config
        .as_object_mut()
        .ok_or_else(|| CeError::Runtime("opencode.json root must be an object".into()))?;

    let entry_str = loader_path.display().to_string();
    let plugins = root
        .entry("plugin")
        .or_insert_with(|| serde_json::json!([]));
    if !plugins.is_array() {
        *plugins = serde_json::json!([]);
    }
    let arr = plugins
        .as_array_mut()
        .ok_or_else(|| CeError::Runtime("`plugin` in opencode.json is not an array".into()))?;

    if !arr.iter().any(|v| v.as_str() == Some(&entry_str)) {
        arr.push(serde_json::Value::String(entry_str));
        changed = true;
    }

    if changed {
        if let Some(parent) = config_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&config)
            .map_err(|e| CeError::Runtime(format!("failed to serialize opencode.json: {e}")))?;
        crate::state::write_atomic(&config_file, serialized.as_bytes())?;
    }

    Ok(changed)
}

/// Surgically removes the managed OpenCode plugin loader from `opencode.json`
/// and deletes the loader file, preserving all custom user plugins and settings.
/// If `opencode.json` contains no remaining user configuration after stripping
/// managed entries, the file is cleanly removed.
/// Returns `Ok(true)` if something was removed, `Ok(false)` otherwise.
pub fn remove_session_start_plugin(config_dir: &Path) -> Result<bool, CeError> {
    let mut changed = false;
    let loader_path = plugin_entry(config_dir);
    if loader_path.exists() {
        crate::state::report_best_effort_remove(&loader_path, std::fs::remove_file(&loader_path));
        changed = true;
    }

    let config_file = config_dir.join("opencode.json");
    if config_file.exists() {
        if let Ok(text) = std::fs::read_to_string(&config_file) {
            if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&text) {
                let entry_str = loader_path.display().to_string();
                if let Some(plugins) = config.get_mut("plugin").and_then(|p| p.as_array_mut()) {
                    let prev_len = plugins.len();
                    plugins.retain(|v| v.as_str() != Some(&entry_str));
                    if plugins.len() != prev_len {
                        changed = true;
                    }
                }
                if config
                    .get("plugin")
                    .and_then(|p| p.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(false)
                {
                    if let Some(obj) = config.as_object_mut() {
                        obj.remove("plugin");
                        changed = true;
                    }
                }

                let skills_dir_str = skills_path(config_dir).display().to_string();
                if let Some(skills_obj) = config.get_mut("skills").and_then(|s| s.as_object_mut()) {
                    if let Some(paths) = skills_obj.get_mut("paths").and_then(|p| p.as_array_mut())
                    {
                        let prev_len = paths.len();
                        paths.retain(|v| v.as_str() != Some(&skills_dir_str));
                        if paths.len() != prev_len {
                            changed = true;
                        }
                    }
                    if skills_obj
                        .get("paths")
                        .and_then(|p| p.as_array())
                        .map(|a| a.is_empty())
                        .unwrap_or(false)
                    {
                        skills_obj.remove("paths");
                        changed = true;
                    }
                }
                if config
                    .get("skills")
                    .and_then(|s| s.as_object())
                    .map(|o| o.is_empty())
                    .unwrap_or(false)
                {
                    if let Some(obj) = config.as_object_mut() {
                        obj.remove("skills");
                        changed = true;
                    }
                }

                if let Some(agent_obj) = config.get_mut("agent").and_then(|a| a.as_object_mut()) {
                    for slot in crate::harness::agents::CE_AGENT_SLOTS {
                        if agent_obj.remove(slot).is_some() {
                            changed = true;
                        }
                    }
                    if agent_obj.is_empty() {
                        if let Some(obj) = config.as_object_mut() {
                            obj.remove("agent");
                            changed = true;
                        }
                    }
                }

                if let Some(mcp_servers) =
                    config.get_mut("mcpServers").and_then(|m| m.as_object_mut())
                {
                    for companion in &["codegraph", "engram", "context7", "rtk"] {
                        if mcp_servers.remove(*companion).is_some() {
                            changed = true;
                        }
                    }
                    if mcp_servers.is_empty() {
                        if let Some(obj) = config.as_object_mut() {
                            obj.remove("mcpServers");
                            changed = true;
                        }
                    }
                }

                if config.as_object().map(|o| o.is_empty()).unwrap_or(false) {
                    crate::state::report_best_effort_remove(
                        &config_file,
                        std::fs::remove_file(&config_file),
                    );
                    return Ok(true);
                }

                if changed {
                    let serialized = serde_json::to_string_pretty(&config).map_err(|e| {
                        CeError::Runtime(format!("failed to serialize opencode.json: {e}"))
                    })?;
                    crate::state::write_atomic(&config_file, serialized.as_bytes())?;
                }
            }
        }
    }

    Ok(changed)
}

#[cfg(test)]
#[path = "tests/plugins.rs"]
mod tests;
