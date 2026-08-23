//! `ce-ai tools` companion registry, version freshness validation, and 24h TTL cache.

use std::collections::BTreeMap;
use std::process::Command;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::commands::Context;
use crate::error::CeError;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(
            parse_version_token("engram 1.2.0"),
            Some("1.2.0".to_string())
        );
        assert_eq!(
            parse_version_token("codegraph v0.5.0"),
            Some("0.5.0".to_string())
        );
        assert_eq!(parse_version_token("no-version-here"), None);
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_version_at_least("1.2.0", "1.2.0"));
        assert!(is_version_at_least("1.3.0", "1.2.0"));
        assert!(!is_version_at_least("1.1.9", "1.2.0"));
    }

    #[test]
    fn test_evaluate_freshness() {
        assert_eq!(
            evaluate_freshness(Some("1.2.0"), "1.2.0"),
            FreshnessStatus::Ok {
                version: "1.2.0".into()
            }
        );
        assert_eq!(
            evaluate_freshness(Some("1.0.0"), "1.2.0"),
            FreshnessStatus::Outdated {
                current: "1.0.0".into(),
                expected: "1.2.0".into()
            }
        );
        assert_eq!(evaluate_freshness(None, "1.2.0"), FreshnessStatus::Missing);
    }
}
