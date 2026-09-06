//! Custom harness fallback mode adapter implementation (--harness custom).
//!
//! Single path contract: `<home>/.ce-ai/custom_harness.json` describes where
//! CE plugin assets, skill folders, and an optional managed rules block live.
//! No other hardcoded custom-mode path may exist in the codebase.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::commands::init_prj::{
    compute_sha256, render_block_content, BLOCK_BEGIN_MARKER, BLOCK_END_MARKER, BLOCK_VERSION,
};
use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::state::AdoptionTier;

/// File name of the single-contract custom-mode configuration.
pub const CONFIG_FILE_NAME: &str = "custom_harness.json";

/// Configuration state for custom fallback harnesses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomHarnessConfig {
    pub plugins_dir: PathBuf,
    pub skills_dir: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_file: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_file: Option<PathBuf>,
}

/// Explicit CLI flag overrides for custom-mode configuration resolution.
#[derive(Debug, Default, Clone)]
pub struct CustomConfigFlags {
    pub plugins_dir: Option<PathBuf>,
    pub skills_dir: Option<PathBuf>,
    pub rules_file: Option<PathBuf>,
    pub mcp_file: Option<PathBuf>,
}

impl CustomHarnessConfig {
    /// Absolute path of the persisted custom-mode config file.
    pub fn config_path(home: &Path) -> PathBuf {
        home.join(".ce-ai").join(CONFIG_FILE_NAME)
    }

    /// Loads the persisted custom-mode config; `Ok(None)` when absent.
    /// Malformed JSON is a hard `CeError::Runtime`, never silently ignored.
    pub fn load_from_home(home: &Path) -> Result<Option<Self>, CeError> {
        let path = Self::config_path(home);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)?;
        serde_json::from_slice(&bytes).map(Some).map_err(|e| {
            CeError::Runtime(format!(
                "invalid {CONFIG_FILE_NAME} at {}: {e}",
                path.display()
            ))
        })
    }

    /// Atomically persists this configuration to `<home>/.ce-ai/custom_harness.json`.
    pub fn save(&self, home: &Path) -> Result<(), CeError> {
        let path = Self::config_path(home);
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| {
            CeError::Runtime(format!(
                "failed to serialize {CONFIG_FILE_NAME} at {}: {e}",
                path.display()
            ))
        })?;
        crate::state::write_atomic(&path, &bytes)
    }

    /// Resolves the effective configuration with flag ▸ config-file
    /// precedence, `~` expansion, and relative-path anchoring. Fails fast
    /// with `CeError::Usage` when `plugins_dir`/`skills_dir` are unresolvable.
    pub fn resolve(home: &Path, flags: &CustomConfigFlags) -> Result<Self, CeError> {
        let from_file = Self::load_from_home(home)?;
        let plugins_dir = flags
            .plugins_dir
            .clone()
            .or_else(|| from_file.as_ref().map(|c| c.plugins_dir.clone()));
        let skills_dir = flags
            .skills_dir
            .clone()
            .or_else(|| from_file.as_ref().map(|c| c.skills_dir.clone()));
        let rules_file = flags
            .rules_file
            .clone()
            .or_else(|| from_file.as_ref().and_then(|c| c.rules_file.clone()));
        let mcp_file = flags
            .mcp_file
            .clone()
            .or_else(|| from_file.as_ref().and_then(|c| c.mcp_file.clone()));

        if plugins_dir.is_none() || skills_dir.is_none() {
            return Err(CeError::Usage(format!(
                "--harness custom requires target directories: pass --plugins-dir and \
                 --skills-dir, or persist them in {} as \
                 {{\"plugins_dir\": \"...\", \"skills_dir\": \"...\", \"rules_file\": \"...\"}}",
                Self::config_path(home).display()
            )));
        }

        Ok(Self {
            plugins_dir: absolutize(expand_tilde(plugins_dir.unwrap_or_default(), home)),
            skills_dir: absolutize(expand_tilde(skills_dir.unwrap_or_default(), home)),
            rules_file: rules_file.map(|p| absolutize(expand_tilde(p, home))),
            mcp_file: mcp_file.map(|p| absolutize(expand_tilde(p, home))),
        })
    }

    /// Restores a config previously embedded in a state entry's `custom` key.
    pub fn from_state_json(value: &serde_json::Value) -> Option<Self> {
        let dir = |key: &str| -> Option<PathBuf> {
            value[key]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        };
        Some(Self {
            plugins_dir: dir("plugins_dir")?,
            skills_dir: dir("skills_dir")?,
            rules_file: dir("rules_file"),
            mcp_file: dir("mcp_file"),
        })
    }

    /// Serializes this config for embedding in a state entry's `custom` key.
    pub fn to_state_json(&self) -> serde_json::Value {
        serde_json::json!({
            "plugins_dir": self.plugins_dir.display().to_string(),
            "skills_dir": self.skills_dir.display().to_string(),
            "rules_file": self.rules_file.as_ref().map(|p| p.display().to_string()),
            "mcp_file": self.mcp_file.as_ref().map(|p| p.display().to_string()),
        })
    }
}

/// Maps a managed rel path onto its location under `plugins_dir`
/// (`plugins/x` → `Some("x")`).
pub fn plugin_rel(managed_rel: &str) -> Option<&str> {
    managed_rel.strip_prefix("plugins/")
}

/// Maps a managed rel path onto its location under `skills_dir`
/// (`skills/x` → `Some("x")`).
pub fn skill_rel(managed_rel: &str) -> Option<&str> {
    managed_rel.strip_prefix("skills/")
}

/// Expands a leading `~` against `home`.
fn expand_tilde(p: PathBuf, home: &Path) -> PathBuf {
    match p.to_str() {
        Some("~") => home.to_path_buf(),
        Some(s) if s.starts_with("~/") => home.join(&s[2..]),
        _ => p,
    }
}

/// Anchors a relative path against the process CWD so persisted snapshots
/// stay stable across invocations. Rooted paths (e.g. `\x` or `/x`, which
/// lack a drive letter on Windows) are preserved verbatim instead of being
/// joined onto the CWD.
fn absolutize(p: PathBuf) -> PathBuf {
    if p.is_absolute() || p.has_root() {
        p
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

/// Renders the current-version managed CE block for markdown rules files.
fn render_full_block() -> String {
    let inner = render_block_content(AdoptionTier::Full);
    format!(
        "<!-- ce-ai:block begin v={} tier=full sha256={} -->\n{}\n{}",
        BLOCK_VERSION,
        compute_sha256(inner),
        inner,
        BLOCK_END_MARKER
    )
}

/// Ensures the markdown rules file contains exactly one current managed CE
/// block, preserving every non-managed byte. Idempotent. Returns `Ok(true)`
/// when the file content changed.
pub fn ensure_rules_block(path: &Path) -> Result<bool, CeError> {
    let existing = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };
    let full_block = render_full_block();
    let updated = if let Some(start_idx) = existing.find(BLOCK_BEGIN_MARKER) {
        let Some(end_rel) = existing[start_idx..].find(BLOCK_END_MARKER) else {
            return Err(CeError::Runtime(format!(
                "malformed managed block in '{}': begin marker without end marker",
                path.display()
            )));
        };
        let end_idx = start_idx + end_rel + BLOCK_END_MARKER.len();
        format!(
            "{}{}{}",
            &existing[..start_idx],
            full_block,
            &existing[end_idx..]
        )
    } else if existing.is_empty() {
        format!("{full_block}\n")
    } else {
        let mut appended = existing.clone();
        if !appended.ends_with('\n') {
            appended.push('\n');
        }
        appended.push('\n');
        appended.push_str(&full_block);
        appended.push('\n');
        appended
    };

    if updated == existing {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    crate::state::write_atomic(path, updated.as_bytes())?;
    Ok(true)
}

/// Removes the managed CE block from a markdown rules file while preserving
/// every other byte. Returns `Ok(false)` when no block is present; a begin
/// marker without an end marker is a hard error.
pub fn strip_rules_block(path: &Path) -> Result<bool, CeError> {
    if !path.exists() {
        return Ok(false);
    }
    let text = std::fs::read_to_string(path)?;
    let Some(start_idx) = text.find(BLOCK_BEGIN_MARKER) else {
        return Ok(false);
    };
    let Some(end_rel) = text[start_idx..].find(BLOCK_END_MARKER) else {
        return Err(CeError::Runtime(format!(
            "malformed managed block in '{}': begin marker without end marker",
            path.display()
        )));
    };
    let end_idx = start_idx + end_rel + BLOCK_END_MARKER.len();

    let head = text[..start_idx].trim_end_matches(['\r', '\n']);
    let tail_content = text[end_idx..].trim_start_matches(['\r', '\n']);
    let mut out = String::with_capacity(text.len());
    if !head.is_empty() {
        out.push_str(head);
        out.push('\n');
    }
    out.push_str(tail_content);

    if out == text {
        return Ok(false);
    }
    crate::state::write_atomic(path, out.as_bytes())?;
    Ok(true)
}

/// Removes now-empty directories left behind after surgical uninstall,
/// stopping at (but never removing) the configured boundary roots.
pub fn prune_empty_dirs(start: &Path, boundaries: &[&Path]) {
    let mut current = start.to_path_buf();
    loop {
        if boundaries.iter().any(|b| current == *b) || current.parent().is_none() {
            break;
        }
        if std::fs::remove_dir(&current).is_err() {
            break;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
}

#[derive(Debug)]
pub struct CustomAdapter {
    config: Option<CustomHarnessConfig>,
}

impl CustomAdapter {
    pub fn new(config: Option<CustomHarnessConfig>) -> Self {
        Self { config }
    }

    /// The resolved configuration, when one was supplied at construction.
    pub fn config(&self) -> Option<&CustomHarnessConfig> {
        self.config.as_ref()
    }
}

impl HarnessAdapter for CustomAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Custom
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if let Some(cfg) = &self.config {
            if let Some(mcp) = &cfg.mcp_file {
                return mcp.clone();
            }
        }
        CustomHarnessConfig::config_path(home)
    }
}

/// Merge and register an MCP server into the custom harness's MCP file using standard `mcpServers` schema.
pub fn register_custom_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &std::collections::BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            serde_json::json!({ "mcpServers": {} })
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse custom MCP config at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        serde_json::json!({ "mcpServers": {} })
    };

    if !config.is_object() {
        return Err(CeError::Runtime(format!(
            "Custom MCP config at {} must be a JSON object",
            config_path.display()
        )));
    }

    let server_def = serde_json::json!({
        "command": command,
        "args": args,
        "env": env,
    });

    match config.get_mut("mcpServers") {
        None => {
            config["mcpServers"] = serde_json::json!({ name: server_def });
        }
        Some(serde_json::Value::Object(map)) => {
            map.insert(name.to_string(), server_def);
        }
        Some(_) => {
            return Err(CeError::Runtime(format!(
                "`mcpServers` in {} must be an object",
                config_path.display()
            )));
        }
    }

    let bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to serialize custom MCP config at {}: {e}",
            config_path.display()
        ))
    })?;

    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    crate::state::write_atomic(config_path, &bytes)
}

/// Registers companion MCP servers (`codegraph`, `engram`) into the custom MCP file.
pub fn register_companions(mcp_file: &Path) -> Result<(), CeError> {
    let env = std::collections::BTreeMap::new();
    register_custom_mcp_server(mcp_file, "codegraph", "codegraph", &["mcp"], &env)?;
    register_custom_mcp_server(mcp_file, "engram", "engram", &["serve"], &env)?;
    Ok(())
}

/// Unregisters companion MCP servers (`codegraph`, `engram`) from the custom MCP file.
pub fn unregister_companions(mcp_file: &Path) -> Result<(), CeError> {
    unregister_custom_mcp_server(mcp_file, "codegraph")?;
    unregister_custom_mcp_server(mcp_file, "engram")?;
    Ok(())
}

/// Removes an MCP server definition from the custom harness's MCP file.
pub fn unregister_custom_mcp_server(config_path: &Path, name: &str) -> Result<bool, CeError> {
    if !config_path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(config_path)?;
    if content.trim().is_empty() {
        return Ok(false);
    }
    let mut config: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse custom MCP config at {}: {e}",
            config_path.display()
        ))
    })?;

    if !config.is_object() {
        return Err(CeError::Runtime(format!(
            "Custom MCP config at {} must be a JSON object",
            config_path.display()
        )));
    }

    if let Some(mcp_val) = config.get("mcpServers") {
        if !mcp_val.is_object() {
            return Err(CeError::Runtime(format!(
                "`mcpServers` in {} must be an object",
                config_path.display()
            )));
        }
    }

    if let Some(mcp_servers) = config.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
        if mcp_servers.remove(name).is_some() {
            if mcp_servers.is_empty() {
                if let Some(obj) = config.as_object_mut() {
                    obj.remove("mcpServers");
                }
            }
            let bytes = serde_json::to_vec_pretty(&config).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to serialize custom MCP config at {}: {e}",
                    config_path.display()
                ))
            })?;
            crate::state::write_atomic(config_path, &bytes)?;
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
#[path = "tests/custom.rs"]
mod tests;
