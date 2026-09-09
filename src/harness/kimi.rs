//! Kimi Code CLI AI harness adapter implementation.
//! Handles Kimi Code CLI's native `~/.kimi-code/mcp.json` (`mcpServers` JSON object schema)
//! and `AGENTS.md` instruction file.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::write_atomic;

#[derive(Debug, Default)]
pub struct KimiAdapter;

impl HarnessAdapter for KimiAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Kimi
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("mcp.json") {
            return home.to_path_buf();
        }

        if let Some(config_env) = std::env::var_os("KIMI_CODE_HOME") {
            return PathBuf::from(config_env).join("mcp.json");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".kimi-code") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".kimi-code").join("mcp.json")
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct KimiMcpConfig {
    #[serde(
        default,
        rename = "mcpServers",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub mcp_servers: BTreeMap<String, KimiMcpServer>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct KimiMcpServer {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Merge and register an MCP server into Kimi Code CLI's `mcp.json` config using native `mcpServers` JSON object schema.
pub fn register_kimi_mcp_server(
    config_path: &Path,
    name: &str,
    command: &str,
    args: &[&str],
    env: &BTreeMap<String, String>,
) -> Result<(), CeError> {
    let mut config: KimiMcpConfig = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if content.trim().is_empty() {
            KimiMcpConfig::default()
        } else {
            serde_json::from_str(&content).map_err(|e| {
                CeError::Runtime(format!(
                    "Failed to parse Kimi mcp.json at {}: {e}",
                    config_path.display()
                ))
            })?
        }
    } else {
        KimiMcpConfig::default()
    };

    let server_entry = config
        .mcp_servers
        .entry(name.to_string())
        .or_insert_with(|| KimiMcpServer {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            env: env.clone(),
            extra: serde_json::Map::new(),
        });

    server_entry.command = command.to_string();
    server_entry.args = args.iter().map(|s| s.to_string()).collect();
    server_entry.env = env.clone();

    let updated_json = serde_json::to_string_pretty(&config)
        .map_err(|e| CeError::Runtime(format!("Failed to serialize Kimi mcp.json: {e}")))?;

    write_atomic(config_path, updated_json.as_bytes())
}

/// Remove an MCP server from Kimi Code CLI's `mcp.json` config.
pub fn unregister_kimi_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError> {
    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;

    if content.trim().is_empty() {
        return Ok(());
    }

    let mut config: KimiMcpConfig = serde_json::from_str(&content).map_err(|e| {
        CeError::Runtime(format!(
            "Failed to parse Kimi mcp.json at {}: {e}",
            config_path.display()
        ))
    })?;

    config.mcp_servers.remove(name);

    let updated_json = serde_json::to_string_pretty(&config)
        .map_err(|e| CeError::Runtime(format!("Failed to serialize Kimi mcp.json: {e}")))?;

    write_atomic(config_path, updated_json.as_bytes())
}

/// Divergence finding between Kimi Code CLI's native plugin manager
/// (`~/.kimi-code/plugins/installed.json`) and ce-ai's managed installation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiMarketplaceDivergence {
    pub plugin_id: String,
    pub native_version: String,
    pub ce_version: String,
}

/// Warn-level finding: a ce-ai managed tree exists under the Kimi harness dir
/// but is not referenced by Kimi's native `config.toml` (`extra_skill_dirs`),
/// so its skills are inactive for Kimi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiOrphanManagedTree {
    pub managed_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct NativeInstalledJson {
    #[serde(default)]
    plugins: Vec<NativeInstalledPlugin>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct NativeInstalledPlugin {
    #[serde(default)]
    id: String,
    #[serde(default)]
    root: Option<PathBuf>,
    #[serde(default)]
    enabled: bool,
}

/// Resolves the native plugin version from `<root>/package.json`
/// (fallback `<root>/plugin.json`).
fn native_plugin_version(root: &Path) -> Option<String> {
    for name in ["package.json", "plugin.json"] {
        let Ok(content) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };
        if let Some(version) = value.get("version").and_then(|v| v.as_str()) {
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}

/// Checks for version divergence between Kimi Code CLI's native plugin manager
/// and the ce-ai managed harness. Returns applicable divergences for enabled
/// native `compound-engineering` plugins only; degrades to an empty vector
/// whenever any inspected file is missing or malformed.
pub fn check_kimi_marketplace_divergence(
    state: &crate::state::state::State,
    cwd: &Path,
    kimi_dir: &Path,
) -> Vec<KimiMarketplaceDivergence> {
    let kimi_entry = state.installed_harnesses.iter().find(|h| {
        if h.get("name").and_then(|n| n.as_str()) != Some("kimi") {
            return false;
        }
        match h.get("scope").and_then(|s| s.as_str()) {
            Some("workspace") => h
                .get("target_dir")
                .and_then(|d| d.as_str())
                .map(|d| {
                    let p = Path::new(d);
                    cwd == p || cwd.starts_with(p)
                })
                .unwrap_or(true),
            Some("global") | None => true,
            _ => false,
        }
    });

    let Some(kimi_entry) = kimi_entry else {
        return Vec::new();
    };

    let ce_version = kimi_entry
        .get("version")
        .and_then(|v| v.as_str())
        .or_else(|| state.release_provenance.as_ref().map(|p| p.tag.as_str()));

    let Some(ce_version) = ce_version else {
        return Vec::new();
    };

    let installed_path = kimi_dir.join("plugins").join("installed.json");
    let Ok(content) = std::fs::read_to_string(&installed_path) else {
        return Vec::new();
    };
    let Ok(registry) = serde_json::from_str::<NativeInstalledJson>(&content) else {
        return Vec::new();
    };

    let norm_ce = crate::harness::claude::normalize_plugin_version(ce_version);
    let mut divergences = Vec::new();

    for plugin in registry.plugins {
        let is_ce_plugin =
            plugin.id == "compound-engineering" || plugin.id.starts_with("compound-engineering@");
        if !is_ce_plugin || !plugin.enabled {
            continue;
        }

        let Some(root) = plugin.root else {
            continue;
        };
        let Some(native_version) = native_plugin_version(&root) else {
            continue;
        };

        let norm_native = crate::harness::claude::normalize_plugin_version(&native_version);
        if norm_native != norm_ce {
            divergences.push(KimiMarketplaceDivergence {
                plugin_id: plugin.id,
                native_version,
                ce_version: ce_version.to_string(),
            });
        }
    }

    divergences
}

/// Cross-platform "does this config entry point at that directory" check.
///
/// Resolution order: raw `Path` equality, then `canonicalize` on both sides
/// (handles symlinks, Windows `\\?\` prefixes, and mixed separators for
/// existing paths), then a separator/case-normalized string comparison so a
/// hand-written `extra_skill_dirs` entry using forward slashes — or a path
/// that no longer exists and therefore cannot be canonicalized — never
/// produces a false orphan finding.
fn paths_refer_to_same(entry: &Path, target: &Path) -> bool {
    if entry == target {
        return true;
    }
    if let (Ok(a), Ok(b)) = (entry.canonicalize(), target.canonicalize()) {
        if a == b {
            return true;
        }
    }
    let normalize = |p: &Path| {
        let mut s = p.to_string_lossy().replace('\\', "/");
        while s.ends_with('/') {
            s.pop();
        }
        if cfg!(windows) {
            s.to_lowercase()
        } else {
            s
        }
    };
    normalize(entry) == normalize(target)
}

/// Detects a ce-ai managed tree under the Kimi harness dir that Kimi's native
/// configuration does not reference via `extra_skill_dirs`. Returns `Some`
/// when the managed tree exists (marked by `install-manifest.json`) and no
/// `extra_skill_dirs` entry points at it; degrades gracefully on unreadable
/// or malformed `config.toml` (treated as not referenced).
pub fn check_kimi_orphan_managed_tree(kimi_dir: &Path) -> Option<KimiOrphanManagedTree> {
    let managed_dir = kimi_dir.join("compound-engineering");
    if !managed_dir.join("install-manifest.json").exists() {
        return None;
    }

    let referenced = std::fs::read_to_string(kimi_dir.join("config.toml"))
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .and_then(|table| table.get("extra_skill_dirs").cloned())
        .and_then(|dirs| dirs.as_array().cloned())
        .map(|dirs| {
            dirs.iter()
                .filter_map(|d| d.as_str())
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let is_referenced = referenced
        .iter()
        .any(|entry| paths_refer_to_same(entry, &managed_dir));

    if is_referenced {
        None
    } else {
        Some(KimiOrphanManagedTree { managed_dir })
    }
}

#[cfg(test)]
#[path = "tests/kimi.rs"]
mod tests;
