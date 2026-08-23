//! `ce-ai audit`: multi-harness, capability-based audit engine for token efficiency
//! and context quality.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;

#[derive(clap::Args, Debug, Default)]
pub struct Args {
    /// Render machine-readable JSON output.
    #[arg(long)]
    pub json: bool,
    /// Exit with non-zero code if score falls below specified percentage (0-100).
    #[arg(long)]
    pub fail_under: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditStatus {
    Pass,
    Warn,
    Fail,
    Info,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditStatus::Pass => write!(f, "PASS"),
            AuditStatus::Warn => write!(f, "WARN"),
            AuditStatus::Fail => write!(f, "FAIL"),
            AuditStatus::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditCheck {
    pub id: String,
    pub category: String,
    pub status: AuditStatus,
    pub satisfied_by: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub harnesses_detected: Vec<String>,
    pub checks: Vec<AuditCheck>,
    pub score_percentage: u32,
    pub pass_count: usize,
    pub warn_count: usize,
    pub fail_count: usize,
}

impl AuditReport {
    pub fn compute_score(checks: &[AuditCheck]) -> (u32, usize, usize, usize) {
        let mut pass_count = 0;
        let mut warn_count = 0;
        let mut fail_count = 0;
        let mut total_applicable = 0;

        for check in checks {
            match check.status {
                AuditStatus::Pass => {
                    pass_count += 1;
                    total_applicable += 1;
                }
                AuditStatus::Warn => {
                    warn_count += 1;
                    total_applicable += 1;
                }
                AuditStatus::Fail => {
                    fail_count += 1;
                    total_applicable += 1;
                }
                AuditStatus::Info => {}
            }
        }

        if total_applicable == 0 {
            return (100, pass_count, warn_count, fail_count);
        }

        let score_pts = (pass_count as f64 * 1.0) + (warn_count as f64 * 0.5);
        let pct = ((score_pts / total_applicable as f64) * 100.0).round() as u32;
        (pct, pass_count, warn_count, fail_count)
    }
}

#[allow(dead_code)]
pub struct AuditCtx {
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
    pub opencode_config_dir: PathBuf,
    pub repo_root: Option<PathBuf>,
}

impl AuditCtx {
    pub fn from_context(ctx: &Context) -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| ctx.config_dir.clone());

        let repo_root = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .map(PathBuf::from);

        Self {
            home_dir,
            config_dir: ctx.config_dir.clone(),
            opencode_config_dir: ctx.opencode_config_dir.clone(),
            repo_root,
        }
    }
}

pub trait Detector {
    fn detect(&self, ctx: &AuditCtx, harnesses: &[HarnessKind]) -> Vec<AuditCheck>;
}

struct CodeIntelDetector;
impl Detector for CodeIntelDetector {
    fn detect(&self, ctx: &AuditCtx, _harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let mut checks = Vec::new();
        if let Some(root) = &ctx.repo_root {
            let codegraph = root.join(".codegraph");
            if codegraph.exists() {
                checks.push(AuditCheck {
                    id: "code-intelligence".into(),
                    category: "repo".into(),
                    status: AuditStatus::Pass,
                    satisfied_by: Some("codegraph".into()),
                    detail: ".codegraph/ index present".into(),
                });
            } else {
                checks.push(AuditCheck {
                    id: "code-intelligence".into(),
                    category: "repo".into(),
                    status: AuditStatus::Warn,
                    satisfied_by: None,
                    detail: ".codegraph/ index not initialized (run 'gentle-ai codegraph init')"
                        .into(),
                });
            }
        }
        checks
    }
}

struct LearningsLibraryDetector;
impl Detector for LearningsLibraryDetector {
    fn detect(&self, ctx: &AuditCtx, _harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let mut checks = Vec::new();
        if let Some(root) = &ctx.repo_root {
            let solutions = root.join("docs").join("solutions");
            if solutions.is_dir() {
                let doc_count = walkdir_md_count(&solutions);
                checks.push(AuditCheck {
                    id: "learnings-library".into(),
                    category: "repo".into(),
                    status: AuditStatus::Pass,
                    satisfied_by: Some("docs/solutions/".into()),
                    detail: format!("docs/solutions/ present ({doc_count} docs)"),
                });
            } else {
                checks.push(AuditCheck {
                    id: "learnings-library".into(),
                    category: "repo".into(),
                    status: AuditStatus::Warn,
                    satisfied_by: None,
                    detail: "docs/solutions/ directory missing".into(),
                });
            }
        }
        checks
    }
}

struct PersistentMemoryDetector;
impl Detector for PersistentMemoryDetector {
    fn detect(&self, ctx: &AuditCtx, _harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let engram_db = ctx.home_dir.join(".engram").join("engram.db");
        let (status, detail) = if engram_db.exists() {
            (
                AuditStatus::Pass,
                "Engram persistent memory server active (~/.engram/engram.db)".into(),
            )
        } else {
            (
                AuditStatus::Warn,
                "Persistent memory database missing (~/.engram/engram.db)".into(),
            )
        };
        vec![AuditCheck {
            id: "persistent-memory".into(),
            category: "grounding".into(),
            status,
            satisfied_by: if engram_db.exists() {
                Some("engram".into())
            } else {
                None
            },
            detail,
        }]
    }
}

struct DocsGroundingDetector;
impl Detector for DocsGroundingDetector {
    fn detect(&self, ctx: &AuditCtx, _harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let opencode_json = ctx.opencode_config_dir.join("opencode.json");
        let has_context7 = std::fs::read_to_string(&opencode_json)
            .map(|t| t.contains("context7"))
            .unwrap_or(false);

        let (status, detail) = if has_context7 {
            (
                AuditStatus::Pass,
                "Context7 tech specs provider configured".into(),
            )
        } else {
            (
                AuditStatus::Info,
                "Context7 tech specs provider not configured (run 'ce-ai tools install context7')"
                    .into(),
            )
        };

        vec![AuditCheck {
            id: "docs-grounding".into(),
            category: "grounding".into(),
            status,
            satisfied_by: if has_context7 {
                Some("context7".into())
            } else {
                None
            },
            detail,
        }]
    }
}

struct CliCompressionDetector;
impl Detector for CliCompressionDetector {
    fn detect(&self, _ctx: &AuditCtx, harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let mut checks = Vec::new();
        let rtk_in_path = is_in_path("rtk");

        for h in harnesses {
            let id = format!("cli-compression/{h}");
            if rtk_in_path {
                checks.push(AuditCheck {
                    id,
                    category: "tokens".into(),
                    status: AuditStatus::Pass,
                    satisfied_by: Some("rtk".into()),
                    detail: "CLI output compressor active".into(),
                });
            } else {
                checks.push(AuditCheck {
                    id,
                    category: "tokens".into(),
                    status: AuditStatus::Info,
                    satisfied_by: None,
                    detail: "CLI compression pre-processor not installed (suggested: 'ce-ai tools install rtk')".into(),
                });
            }
        }
        checks
    }
}

struct McpSprawlDetector;
impl Detector for McpSprawlDetector {
    fn detect(&self, ctx: &AuditCtx, harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let mut checks = Vec::new();
        for h in harnesses {
            let config_file = match h {
                HarnessKind::Opencode => ctx.opencode_config_dir.join("opencode.json"),
                _ => ctx
                    .home_dir
                    .join(".config")
                    .join(h.to_string())
                    .join("config.json"),
            };
            if let Ok(text) = std::fs::read_to_string(&config_file) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let mcp_count = val
                        .get("mcpServers")
                        .or_else(|| val.get("mcp"))
                        .and_then(|v| v.as_object())
                        .map(|m| m.len())
                        .unwrap_or(0);

                    let (status, detail) = if mcp_count > 5 {
                        (AuditStatus::Warn, format!("{mcp_count} MCP servers configured globally (>5 threshold may waste context)"))
                    } else {
                        (
                            AuditStatus::Pass,
                            format!(
                                "{mcp_count} MCP servers configured globally (under threshold)"
                            ),
                        )
                    };

                    checks.push(AuditCheck {
                        id: format!("mcp-sprawl/{h}"),
                        category: "tokens".into(),
                        status,
                        satisfied_by: None,
                        detail,
                    });
                }
            }
        }
        checks
    }
}

struct PromptDuplicationDetector;
impl Detector for PromptDuplicationDetector {
    fn detect(&self, ctx: &AuditCtx, harnesses: &[HarnessKind]) -> Vec<AuditCheck> {
        let mut checks = Vec::new();
        for h in harnesses {
            let config_file = match h {
                HarnessKind::Opencode => ctx.opencode_config_dir.join("opencode.json"),
                _ => ctx
                    .home_dir
                    .join(".config")
                    .join(h.to_string())
                    .join("config.json"),
            };
            if let Ok(text) = std::fs::read_to_string(&config_file) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let mut blocks: BTreeMap<String, usize> = BTreeMap::new();
                    if let Some(agent_map) = val.get("agent").and_then(|a| a.as_object()) {
                        for (_agent_name, agent_val) in agent_map {
                            if let Some(prompt) = agent_val.get("prompt").and_then(|p| p.as_str()) {
                                for paragraph in prompt.split("\n\n") {
                                    let trimmed = paragraph.trim();
                                    if trimmed.len() >= 200 {
                                        *blocks.entry(trimmed.to_string()).or_default() += 1;
                                    }
                                }
                            }
                        }
                    }

                    let duplicated: usize = blocks.values().filter(|&&count| count >= 3).sum();
                    if duplicated > 0 {
                        checks.push(AuditCheck {
                            id: format!("prompt-duplication/{h}"),
                            category: "hygiene".into(),
                            status: AuditStatus::Warn,
                            satisfied_by: None,
                            detail: format!("{duplicated} duplicated prompt blocks (>=200 chars across >=3 agents)"),
                        });
                    } else {
                        checks.push(AuditCheck {
                            id: format!("prompt-duplication/{h}"),
                            category: "hygiene".into(),
                            status: AuditStatus::Pass,
                            satisfied_by: None,
                            detail: "No prompt duplication detected across agents".into(),
                        });
                    }
                }
            }
        }
        checks
    }
}

fn walkdir_md_count(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += walkdir_md_count(&path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                count += 1;
            }
        }
    }
    count
}

fn is_in_path(name: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.join(name).is_file() {
                return true;
            }
        }
    }
    false
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let audit_ctx = AuditCtx::from_context(ctx);
    let installed_harnesses = HarnessKind::detect_ce_installed_harnesses(&audit_ctx.home_dir);
    let harness_names: Vec<String> = installed_harnesses.iter().map(|h| h.to_string()).collect();

    let detectors: Vec<Box<dyn Detector>> = vec![
        Box::new(CodeIntelDetector),
        Box::new(LearningsLibraryDetector),
        Box::new(PersistentMemoryDetector),
        Box::new(DocsGroundingDetector),
        Box::new(CliCompressionDetector),
        Box::new(McpSprawlDetector),
        Box::new(PromptDuplicationDetector),
    ];

    let mut checks = Vec::new();
    for detector in detectors {
        checks.extend(detector.detect(&audit_ctx, &installed_harnesses));
    }

    let (score_pct, pass_cnt, warn_cnt, fail_cnt) = AuditReport::compute_score(&checks);

    let report = AuditReport {
        harnesses_detected: harness_names.clone(),
        checks: checks.clone(),
        score_percentage: score_pct,
        pass_count: pass_cnt,
        warn_count: warn_cnt,
        fail_count: fail_cnt,
    };

    if args.json {
        let json_output = serde_json::to_string_pretty(&report)?;
        println!("{json_output}");
    } else {
        println!("== [ce-ai Agent Environment Audit] ==");
        println!("harnesses detected: {}\n", harness_names.join(", "));

        for check in &checks {
            let status_str = match check.status {
                AuditStatus::Pass => "PASS",
                AuditStatus::Warn => "WARN",
                AuditStatus::Fail => "FAIL",
                AuditStatus::Info => "INFO",
            };
            let sat_str = match &check.satisfied_by {
                Some(sat) => format!(" satisfied-by: {sat}"),
                None => String::new(),
            };
            println!(
                "[{:<8}] {:<4} {:<30} {}{}",
                check.category, status_str, check.id, check.detail, sat_str
            );
        }

        println!(
            "\nscore: {}% ({} pass / {} warn / {} fail)",
            score_pct, pass_cnt, warn_cnt, fail_cnt
        );
    }

    if let Some(threshold) = args.fail_under {
        if score_pct < threshold {
            return Err(CeError::Runtime(format!(
                "audit score {}% is below required threshold of {}%",
                score_pct, threshold
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_audit_score_math() {
        let checks = vec![
            AuditCheck {
                id: "c1".into(),
                category: "repo".into(),
                status: AuditStatus::Pass,
                satisfied_by: None,
                detail: "ok".into(),
            },
            AuditCheck {
                id: "c2".into(),
                category: "repo".into(),
                status: AuditStatus::Warn,
                satisfied_by: None,
                detail: "warn".into(),
            },
        ];
        let (score, pass, warn, fail) = AuditReport::compute_score(&checks);
        assert_eq!(score, 75);
        assert_eq!(pass, 1);
        assert_eq!(warn, 1);
        assert_eq!(fail, 0);
    }

    #[test]
    fn test_audit_run_runs_cleanly() {
        let tmp = TempDir::new().unwrap();
        let ctx = Context {
            config_dir: tmp.path().to_path_buf(),
            opencode_config_dir: tmp.path().to_path_buf(),
            dry_run: false,
            verbose: false,
            quiet: true,
        };
        let args = Args::default();
        assert!(run(&ctx, &args).is_ok());
    }
}
