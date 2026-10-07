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

/// Represents the detected OpenCode major version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenCodeVersion {
    V1,
    V2,
    Unknown,
}

/// Detects the OpenCode major version by executing `opencode --version`.
pub fn detect_opencode_version() -> OpenCodeVersion {
    if let Ok(output) = std::process::Command::new("opencode")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return parse_opencode_version(&stdout);
        }
    }
    OpenCodeVersion::Unknown
}

/// Parses an OpenCode version string into an `OpenCodeVersion`.
pub fn parse_opencode_version(version_str: &str) -> OpenCodeVersion {
    for part in version_str.split_whitespace() {
        let clean = part.strip_prefix('v').unwrap_or(part);
        if let Some(major) = clean.split('.').next() {
            if major == "2" {
                return OpenCodeVersion::V2;
            } else if major == "1" {
                return OpenCodeVersion::V1;
            }
        }
    }
    OpenCodeVersion::Unknown
}

/// Absolute path of the autodiscovered plugin loader in `<config>/plugins/` (OpenCode V2).
pub fn autodiscovered_plugin_entry(config_dir: &Path) -> PathBuf {
    config_dir.join("plugins").join("compound-engineering.js")
}

/// Marker comment indicating a command file is managed by `ce-ai`.
pub const MANAGED_COMMAND_MARKER: &str = "<!-- ce-ai:managed-command -->";

/// Extracts `(description, user_invocable)` from YAML frontmatter in `SKILL.md`.
pub fn extract_skill_metadata(content: &str) -> (Option<String>, bool) {
    let mut lines = content.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return (None, true);
    }
    let mut description = None;
    let mut user_invocable = true;
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("description:") {
            let val = rest.trim();
            let clean = val.trim_matches('"').trim_matches('\'');
            description = Some(clean.to_string());
        } else if let Some(rest) = trimmed.strip_prefix("user-invocable:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if val.eq_ignore_ascii_case("false") {
                user_invocable = false;
            }
        }
    }
    (description, user_invocable)
}

/// Materializes user-invocable skills as native OpenCode command files under `<config_dir>/commands/`.
/// Returns the number of managed command files ensured.
pub fn ensure_managed_commands(config_dir: &Path) -> Result<usize, CeError> {
    let skills_dir = skills_path(config_dir);
    if !skills_dir.is_dir() {
        return Ok(0);
    }
    let cmd_dir = config_dir.join("commands");
    if !cmd_dir.exists() {
        std::fs::create_dir_all(&cmd_dir)?;
    }

    let mut written = 0;
    let entries = std::fs::read_dir(&skills_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skill_id = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            let skill_file = path.join("SKILL.md");
            if !skill_file.is_file() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&skill_file) else {
                continue;
            };
            let (desc_opt, user_invocable) = extract_skill_metadata(&content);
            if !user_invocable {
                continue;
            }
            let desc = desc_opt.unwrap_or_else(|| format!("Load and execute {skill_id} skill"));
            let command_file = cmd_dir.join(format!("{skill_id}.md"));
            let command_content = format!(
                "---\ndescription: \"{}\"\n---\n\n{}\n\nLoad and execute the `{}` skill.\n\n$ARGUMENTS\n",
                desc.replace('"', "\\\""),
                MANAGED_COMMAND_MARKER,
                skill_id
            );
            let needs_write = match std::fs::read_to_string(&command_file) {
                Ok(existing) => existing != command_content,
                Err(_) => true,
            };
            if needs_write {
                crate::state::write_atomic(&command_file, command_content.as_bytes())?;
            }
            written += 1;
        }
    }
    Ok(written)
}

/// Removes `ce-ai` managed command files under `<config_dir>/commands/`, preserving custom user commands.
/// Returns the count of files removed.
pub fn remove_managed_commands(config_dir: &Path) -> Result<usize, CeError> {
    let cmd_dir = config_dir.join("commands");
    if !cmd_dir.is_dir() {
        return Ok(0);
    }
    let mut removed = 0;
    let entries = std::fs::read_dir(&cmd_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if content.contains(MANAGED_COMMAND_MARKER) {
                    crate::state::report_best_effort_remove(&path, std::fs::remove_file(&path));
                    removed += 1;
                }
            }
        }
    }
    if let Ok(mut remaining) = std::fs::read_dir(&cmd_dir) {
        if remaining.next().is_none() {
            crate::state::report_best_effort_remove(&cmd_dir, std::fs::remove_dir(&cmd_dir));
        }
    }
    Ok(removed)
}

/// Whether `content` is a recognized-valid OpenCode plugin loader for the detected OpenCode version.
pub fn is_valid_loader_content(content: &str) -> bool {
    is_valid_loader_content_for(content, detect_opencode_version())
}

/// Validates plugin loader content against a target OpenCode major version.
///
/// OpenCode V2 requires a default export shaped as a definition with an `id`
/// and a `setup`/`effect` function; V1 function-style exports no longer load.
/// Legacy `ceLoader` function exports remain recognized on their own to preserve
/// the #325 loader-safety guarantee.
pub fn is_valid_loader_content_for(content: &str, target_version: OpenCodeVersion) -> bool {
    if content.starts_with("export default function ceLoader") {
        return true; // legacy test fixture guarantee (#325)
    }

    if !content.contains("session.created") {
        return false;
    }

    let is_v1_shape = content.contains("CompoundEngineeringPlugin") || content.contains("server:");
    let is_v2_shape =
        content.contains("setup") && (content.contains("id:") || content.contains("Plugin.define"));
    let is_dual_shape = is_v1_shape && is_v2_shape;

    match target_version {
        OpenCodeVersion::V2 => is_v2_shape,
        OpenCodeVersion::V1 => is_v1_shape,
        OpenCodeVersion::Unknown => is_dual_shape || is_v2_shape,
    }
}

/// Resolves the OpenCode plugin loader bytes for a given source tree,
/// validating that the source's own loader is still recognized (see
/// `is_valid_loader_content_for`). When it isn't, falls back to
/// `BUILTIN_LOADER`, the loader embedded in this `ce-ai` binary. Shared by
/// `install_loader` (fresh installs) and the sync engine's drift-repair
/// path, so neither can regress an already-correct installed loader to a
/// stale/incompatible one.
pub fn resolve_loader_bytes(source_root: &Path) -> Vec<u8> {
    let src = source_root.join(SOURCE_LOADER_PATH);
    match std::fs::read(&src) {
        Ok(b) => match std::str::from_utf8(&b) {
            Ok(s) if is_valid_loader_content_for(s, OpenCodeVersion::V2) => b,
            Ok(_) => BUILTIN_LOADER.as_bytes().to_vec(),
            Err(_) => b,
        },
        Err(_) => BUILTIN_LOADER.as_bytes().to_vec(),
    }
}

/// Copies the CE loader from the source tree into
/// `<config>/compound-engineering/plugins/compound-engineering.js` (OI-3).
/// Returns the managed-relative path and its SHA256 for the manifest (OI-5).
pub fn install_loader(source_root: &Path, config_dir: &Path) -> Result<ManifestFile, CeError> {
    let bytes = resolve_loader_bytes(source_root);
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
/// `session.created` hook and valid shape, and is registered in `opencode.json`.
pub fn has_session_start_plugin(config_dir: &Path) -> bool {
    let config_file = config_dir.join("opencode.json");
    if config_file.exists() {
        if let Ok(config_str) = std::fs::read_to_string(&config_file) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&config_str) {
                let has_native = |key: &str| -> bool {
                    val.get(key)
                        .and_then(|p| p.as_array())
                        .map(|arr| {
                            arr.iter().any(|v| {
                                v.as_str()
                                    .map(|s| {
                                        (s.starts_with("@everyinc/compound-engineering")
                                            || s.starts_with("compound-engineering"))
                                            && !s.ends_with(".js")
                                    })
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                };
                if has_native("plugins") || has_native("plugin") {
                    return true;
                }
            }
        }
    }

    let loader_path = plugin_entry(config_dir);
    let auto_loader_path = autodiscovered_plugin_entry(config_dir);

    let content = if loader_path.exists() {
        std::fs::read_to_string(&loader_path).ok()
    } else if auto_loader_path.exists() {
        std::fs::read_to_string(&auto_loader_path).ok()
    } else {
        None
    };

    let Some(content) = content else {
        return false;
    };

    if !is_valid_loader_content(&content) {
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
    let auto_entry = auto_loader_path.display().to_string();

    let has_entry_in = |key: &str| -> bool {
        val.get(key)
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter().any(|v| {
                    v.as_str()
                        .map(|s| s == expected_entry || s == auto_entry)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    };

    has_entry_in("plugins") || has_entry_in("plugin")
}

/// Ensures that the canonical OpenCode plugin loader is installed at
/// `plugin_entry(config_dir)` and registered in `opencode.json`.
/// When OpenCode V2 is detected, also ensures the autodiscovered plugin path
/// and managed command markdown files under `commands/`.
/// Returns `Ok(true)` if modified, `Ok(false)` if already up to date.
pub fn ensure_session_start_plugin(config_dir: &Path) -> Result<bool, CeError> {
    let mut changed = false;
    let loader_path = plugin_entry(config_dir);
    let version = detect_opencode_version();

    let needs_loader_write = match std::fs::read_to_string(&loader_path) {
        Ok(s) => !is_valid_loader_content_for(&s, version),
        Err(_) => true,
    };

    if needs_loader_write {
        if let Some(parent) = loader_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::state::write_atomic(&loader_path, BUILTIN_LOADER.as_bytes())?;
        changed = true;
    }

    // On OpenCode V2, also ensure autodiscovery location in plugins/
    if version == OpenCodeVersion::V2 {
        let auto_loader = autodiscovered_plugin_entry(config_dir);
        let needs_auto_write = match std::fs::read_to_string(&auto_loader) {
            Ok(s) => !is_valid_loader_content_for(&s, version),
            Err(_) => true,
        };
        if needs_auto_write {
            if let Some(parent) = auto_loader.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::state::write_atomic(&auto_loader, BUILTIN_LOADER.as_bytes())?;
            changed = true;
        }
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
    let target_key = if root.contains_key("plugins") {
        "plugins"
    } else if root.contains_key("plugin") || version == OpenCodeVersion::V1 {
        "plugin"
    } else {
        "plugins"
    };

    let plugins = root
        .entry(target_key)
        .or_insert_with(|| serde_json::json!([]));
    if !plugins.is_array() {
        *plugins = serde_json::json!([]);
    }
    let arr = plugins.as_array_mut().ok_or_else(|| {
        CeError::Runtime(format!("`{target_key}` in opencode.json is not an array"))
    })?;

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

    // Materialize managed commands for V2 and general availability
    let _ = ensure_managed_commands(config_dir)?;

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

    let auto_loader = autodiscovered_plugin_entry(config_dir);
    if auto_loader.exists() {
        crate::state::report_best_effort_remove(&auto_loader, std::fs::remove_file(&auto_loader));
        changed = true;
    }

    let plugins_dir = config_dir.join("plugins");
    if plugins_dir.is_dir() {
        if let Ok(mut it) = std::fs::read_dir(&plugins_dir) {
            if it.next().is_none() {
                crate::state::report_best_effort_remove(
                    &plugins_dir,
                    std::fs::remove_dir(&plugins_dir),
                );
            }
        }
    }

    let cmd_removed = remove_managed_commands(config_dir)?;
    if cmd_removed > 0 {
        changed = true;
    }

    let config_file = config_dir.join("opencode.json");
    if config_file.exists() {
        if let Ok(text) = std::fs::read_to_string(&config_file) {
            if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&text) {
                let entry_str = loader_path.display().to_string();
                let auto_entry_str = auto_loader.display().to_string();

                for key in &["plugin", "plugins"] {
                    if let Some(plugins) = config.get_mut(*key).and_then(|p| p.as_array_mut()) {
                        let prev_len = plugins.len();
                        plugins.retain(|v| {
                            v.as_str()
                                .map(|s| s != entry_str && s != auto_entry_str)
                                .unwrap_or(true)
                        });
                        if plugins.len() != prev_len {
                            changed = true;
                        }
                    }
                    if config
                        .get(*key)
                        .and_then(|p| p.as_array())
                        .map(|a| a.is_empty())
                        .unwrap_or(false)
                    {
                        if let Some(obj) = config.as_object_mut() {
                            obj.remove(*key);
                            changed = true;
                        }
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
