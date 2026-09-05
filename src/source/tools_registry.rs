//! `ce-ai tools` companion registry, version freshness validation, and 24h TTL cache.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::state::State;
use crate::state::write_atomic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FreshnessStatus {
    Ok { version: String },
    Outdated { current: String, expected: String },
    Missing,
    Offline { current: String },
}

impl std::fmt::Display for FreshnessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FreshnessStatus::Ok { version } => write!(f, "v{version} (ok)"),
            FreshnessStatus::Outdated { current, expected } => {
                write!(f, "v{current} (outdated, expected v{expected})")
            }
            FreshnessStatus::Missing => write!(f, "not found"),
            FreshnessStatus::Offline { current } => write!(f, "v{current} (offline)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionToolInfo {
    pub name: String,
    pub label: String,
    pub category: String,
    pub min_version: String,
    pub latest_version: String,
    pub install_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSuggestionInfo {
    pub name: String,
    pub description: String,
    pub resolve_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsRegistryCache {
    pub updated_at: String,
    pub tools: BTreeMap<String, CompanionToolInfo>,
    pub skills: BTreeMap<String, SkillSuggestionInfo>,
}

impl Default for ToolsRegistryCache {
    fn default() -> Self {
        Self::embedded_default()
    }
}

impl ToolsRegistryCache {
    pub fn embedded_default() -> Self {
        let mut tools = BTreeMap::new();
        tools.insert(
            "engram".into(),
            CompanionToolInfo {
                name: "engram".into(),
                label: "Engram Persistent Memory Server".into(),
                category: "MCP Server".into(),
                min_version: "1.2.0".into(),
                latest_version: "1.2.0".into(),
                install_cmd: "ce-ai tools install engram".into(),
            },
        );
        tools.insert(
            "codegraph".into(),
            CompanionToolInfo {
                name: "codegraph".into(),
                label: "CodeGraph Codebase Indexer".into(),
                category: "MCP Server & CLI".into(),
                min_version: "0.5.0".into(),
                latest_version: "0.5.0".into(),
                install_cmd: "ce-ai tools install codegraph".into(),
            },
        );
        tools.insert(
            "context7".into(),
            CompanionToolInfo {
                name: "context7".into(),
                label: "Context7 Tech Specs Provider".into(),
                category: "MCP Server".into(),
                min_version: "1.0.0".into(),
                latest_version: "1.0.0".into(),
                install_cmd: "ce-ai tools install context7".into(),
            },
        );
        tools.insert(
            "rtk".into(),
            CompanionToolInfo {
                name: "rtk".into(),
                label: "RTK CLI Token Reduction Engine".into(),
                category: "CLI Pre-Processor".into(),
                min_version: "0.2.1".into(),
                latest_version: "0.2.1".into(),
                install_cmd: "ce-ai tools install rtk".into(),
            },
        );

        let mut skills = BTreeMap::new();
        skills.insert(
            "sequential-thinking".into(),
            SkillSuggestionInfo {
                name: "sequential-thinking".into(),
                description: "Structured step-by-step reasoning & hypothesis refinement".into(),
                resolve_cmd: "ce-ai skills resolve sequential-thinking".into(),
            },
        );

        Self {
            updated_at: Utc::now().to_rfc3339(),
            tools,
            skills,
        }
    }

    pub fn cache_path(ctx: &Context) -> std::path::PathBuf {
        ctx.config_dir.join("cache").join("companion-registry.json")
    }

    pub fn load_or_default(ctx: &Context) -> Self {
        let path = Self::cache_path(ctx);
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(cached) = serde_json::from_slice::<Self>(&bytes) {
                // Validate 24-hour TTL (86,400 seconds)
                if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&cached.updated_at) {
                    let now = Utc::now();
                    if (now - updated.with_timezone(&Utc)).num_hours() < 24 {
                        return cached;
                    }
                }
            }
        }
        let fresh = Self::embedded_default();
        let _ = fresh.save_cache(ctx);
        fresh
    }

    pub fn save_cache(&self, ctx: &Context) -> Result<(), CeError> {
        let path = Self::cache_path(ctx);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let data = serde_json::to_vec_pretty(self)?;
        write_atomic(&path, &data)
    }
}

/// Extract installed tool version by calling `--version` or checking PATH.
pub fn extract_tool_version(tool_name: &str) -> Option<String> {
    let output = Command::new(tool_name).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    parse_version_token(&text)
}

/// Extract SemVer-like version string from CLI output text.
pub fn parse_version_token(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let clean = token.trim_start_matches('v').trim_end_matches(',');
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.len() >= 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
            return Some(clean.to_string());
        }
    }
    None
}

/// Evaluates FreshnessStatus comparing installed vs registry version.
pub fn evaluate_freshness(installed: Option<&str>, expected: &str) -> FreshnessStatus {
    let Some(inst) = installed else {
        return FreshnessStatus::Missing;
    };
    if is_version_at_least(inst, expected) {
        FreshnessStatus::Ok {
            version: inst.to_string(),
        }
    } else {
        FreshnessStatus::Outdated {
            current: inst.to_string(),
            expected: expected.to_string(),
        }
    }
}

/// Compare two SemVer strings (e.g. "1.2.0" >= "1.1.0").
pub fn is_version_at_least(current: &str, min_expected: &str) -> bool {
    let parse_nums = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v')
            .split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    };
    let curr_nums = parse_nums(current);
    let exp_nums = parse_nums(min_expected);
    curr_nums >= exp_nums
}

/// Collects candidate MCP configuration paths across active and supported harnesses.
pub fn find_mcp_config_paths(ctx: &Context) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path).unwrap_or_default();

    // 1. OpenCode configuration (workspace-resolved or global)
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    paths.push(opencode_dir.join("opencode.json"));
    if opencode_dir != ctx.opencode_config_dir {
        paths.push(ctx.opencode_config_dir.join("opencode.json"));
    }

    // 2. Workspace root configs
    if let Some(ws) = &ctx.workspace_root {
        paths.push(ws.join("opencode.json"));
        paths.push(ws.join(".cursor").join("mcp.json"));
        paths.push(ws.join(".claude.json"));
    }

    // 3. Native harness configurations
    let home_dir = crate::harness::home_dir_from_ctx(ctx);
    for entry in &state.installed_harnesses {
        let Some(name) = entry["name"].as_str() else {
            continue;
        };
        if let Ok(kind) = name.parse::<HarnessKind>() {
            match kind {
                HarnessKind::Cursor => {
                    paths.push(home_dir.join(".cursor").join("mcp.json"));
                }
                HarnessKind::Claude => {
                    paths
                        .push(crate::harness::claude::ClaudeAdapter.default_config_path(&home_dir));
                }
                HarnessKind::Codex => {
                    paths.push(crate::harness::codex::CodexAdapter.default_config_path(&home_dir));
                }
                HarnessKind::Copilot => {
                    paths.push(
                        crate::harness::copilot::CopilotAdapter.default_config_path(&home_dir),
                    );
                }
                HarnessKind::Kimi => {
                    paths.push(crate::harness::kimi::KimiAdapter.default_config_path(&home_dir));
                }
                HarnessKind::Agy => {
                    paths.push(crate::harness::agy::AgyAdapter.default_config_path(&home_dir));
                }
                HarnessKind::Fx => {
                    paths.push(crate::harness::fx::FxAdapter.default_config_path(&home_dir));
                }
                _ => {}
            }
        }
    }

    // Always check common user home configs even if state is fresh/partial
    paths.push(crate::harness::claude::ClaudeAdapter.default_config_path(&home_dir));
    paths.push(home_dir.join(".cursor").join("mcp.json"));

    let mut seen = HashSet::new();
    paths.retain(|p| seen.insert(p.clone()));
    paths
}

/// Checks whether an MCP server matching `name` is registered in any candidate harness configuration.
pub fn is_mcp_server_configured(ctx: &Context, name: &str) -> bool {
    let norm_target = name.to_lowercase();
    let stripped_target = norm_target.replace(['-', '_'], "");

    for path in find_mcp_config_paths(ctx) {
        if !path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if !content.contains("mcpServers") && !content.contains("mcp_servers") {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };

        let mcp_obj = val.get("mcpServers").or_else(|| val.get("mcp_servers"));
        if let Some(serde_json::Value::Object(map)) = mcp_obj {
            for (key, server_val) in map {
                let norm_key = key.to_lowercase();
                let stripped_key = norm_key.replace(['-', '_'], "");

                if norm_key == norm_target || stripped_key == stripped_target {
                    return true;
                }

                if let Some(cmd) = server_val.get("command").and_then(|c| c.as_str()) {
                    if cmd.to_lowercase().contains(&norm_target) {
                        return true;
                    }
                }
                if let Some(serde_json::Value::Array(args)) = server_val.get("args") {
                    for arg in args {
                        if let Some(s) = arg.as_str() {
                            if s.to_lowercase().contains(&norm_target) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

/// Checks whether a skill suggestion is configured either as an MCP server or in the skill registry.
pub fn is_skill_configured(ctx: &Context, name: &str) -> bool {
    if is_mcp_server_configured(ctx, name) {
        return true;
    }
    let registry_path = ctx.config_dir.join("skills-registry.json");
    if let Ok(reg) = crate::source::registry::SkillRegistry::load(&registry_path) {
        if reg.skills.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
            return true;
        }
    }
    if let Some(ws) = &ctx.workspace_root {
        let ws_reg_path = ws.join(".ce-ai").join("skills-registry.json");
        if let Ok(reg) = crate::source::registry::SkillRegistry::load(&ws_reg_path) {
            if reg.skills.iter().any(|s| s.name.eq_ignore_ascii_case(name)) {
                return true;
            }
        }
    }
    false
}

/// Evaluates tool freshness comparing installed CLI version or MCP server presence.
pub fn detect_tool_freshness(
    ctx: &Context,
    tool_name: &str,
    info: &CompanionToolInfo,
) -> FreshnessStatus {
    if let Some(installed_ver) = extract_tool_version(tool_name) {
        return evaluate_freshness(Some(&installed_ver), &info.latest_version);
    }
    if is_mcp_server_configured(ctx, tool_name) {
        return FreshnessStatus::Ok {
            version: info.latest_version.clone(),
        };
    }
    FreshnessStatus::Missing
}

#[cfg(test)]
#[path = "tests/tools_registry.rs"]
mod tests;
