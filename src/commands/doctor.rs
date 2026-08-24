//! `ce-ai doctor`: report config-validity, diff (drift), state-consistency,
//! companion tool readiness, version freshness, and skill health findings.
//! Exits non-zero when any finding exists.

use std::collections::BTreeMap;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::read_config;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::source::tools_registry::{
    evaluate_freshness, extract_tool_version, FreshnessStatus, ToolsRegistryCache,
};
use crate::state::diff::{self, Action};
use crate::state::state::State;

#[derive(clap::Args, Debug, Default)]
pub struct Args {
    /// Enforce strict health checks, failing doctor with non-zero exit code if any tool is outdated.
    #[arg(long)]
    pub strict: bool,
}

/// Extracts the `OWNER/REPO` slug from a GitHub `origin` remote URL
/// (ssh or https forms, optional `.git` suffix). Returns `None` for
/// non-GitHub remotes.
fn github_slug_from_url(url: &str) -> Option<String> {
    let no_git = url.trim().strip_suffix(".git").unwrap_or(url.trim());
    if let Some(rest) = no_git.strip_prefix("git@github.com:") {
        return Some(rest.to_string());
    }
    no_git
        .split_once("github.com/")
        .map(|(_, rest)| rest.trim_end_matches('/').to_string())
}

/// Resolves the GitHub slug for `origin` inside `repo_root`.
fn github_slug_from_remote(repo_root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    github_slug_from_url(&String::from_utf8_lossy(&out.stdout))
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let mut findings: Vec<String> = Vec::new();

    // Config validity: opencode.json must parse (D4).
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    if let Err(err) = read_config(&opencode_json) {
        findings.push(format!("config-invalid: {err}"));
    }

    // Diff: managed files vs the install manifest (SU-3).
    let manifest = InstallManifest::load(&ctx.opencode_config_dir);
    if let Ok(manifest) = &manifest {
        let desired: BTreeMap<String, String> = manifest
            .files
            .iter()
            .map(|f| (f.path.clone(), f.sha256.clone()))
            .collect();
        let managed = ctx.opencode_config_dir.join(MANAGED_DIR);
        for action in diff::diff(&desired, &desired, &managed).actions {
            let (kind, path) = match action {
                Action::Copy { path } => ("missing", path),
                Action::Restore { path } => ("modified", path),
                Action::Remove { path } => ("stale", path),
            };
            findings.push(format!("diff: {kind} {path}"));
        }
    }

    // State consistency: the opencode state entry and the manifest must agree.
    let state = State::load_with_workspace_overrides(
        &ctx.config_dir.join("state.json"),
        ctx.workspace_root.as_deref(),
    )?;
    let has_entry = state
        .installed_harnesses
        .iter()
        .any(|h| h["name"].as_str() == Some("opencode"));
    if has_entry != manifest.is_ok() {
        findings
            .push("state-inconsistent: opencode state entry and install manifest disagree".into());
    }

    // Model assignment drift between state.json and opencode.json (#111).
    if let Ok(config) = read_config(&opencode_json) {
        findings.extend(crate::commands::models::model_drift_findings(
            &state, &config,
        ));
    }

    // Skill Registry Integrity Health Probe
    if let Ok(skill_findings) = crate::source::registry::check_skill_registry_health(ctx) {
        findings.extend(skill_findings);
    }

    // Companion Tools Readiness & Version Freshness Probe (#112)
    let registry = ToolsRegistryCache::load_or_default(ctx);
    for (name, info) in &registry.tools {
        let installed = extract_tool_version(name);
        let freshness = evaluate_freshness(installed.as_deref(), &info.latest_version);

        match freshness {
            FreshnessStatus::Ok { version } => {
                println!("doctor-info: {} v{} (ok)", name, version);
            }
            FreshnessStatus::Outdated { current, expected } => {
                let msg = format!(
                    "tool-outdated: {} v{} is outdated (v{} expected; run '{}')",
                    name, current, expected, info.install_cmd
                );
                if args.strict {
                    findings.push(msg);
                } else {
                    println!("doctor-info: {}", msg);
                }
            }
            FreshnessStatus::Missing => {
                let msg = format!(
                    "companion tool '{}' not found (suggested: '{}')",
                    name, info.install_cmd
                );
                if args.strict {
                    findings.push(format!("tool-missing: {msg}"));
                } else {
                    println!("doctor-info: {}", msg);
                }
            }
            FreshnessStatus::Offline { current } => {
                println!("doctor-info: {} v{} (offline)", name, current);
            }
        }
    }

    // Skill Suggestions Probe (#112)
    for (name, skill) in &registry.skills {
        println!(
            "doctor-info: skill-suggestion: {} (run '{}')",
            name, skill.resolve_cmd
        );
    }

    // Project adoption health checks
    for p in &state.projects {
        let agents_file = p.path.join(&p.file);
        match crate::commands::init_prj::check_adoption_block_status(&agents_file, p.tier) {
            crate::commands::init_prj::AdoptionBlockStatus::Ok => {}
            crate::commands::init_prj::AdoptionBlockStatus::FileMissing => {
                findings.push(format!(
                    "project-adoption: missing instruction file '{}' at '{}'",
                    p.file,
                    p.path.display()
                ));
            }
            crate::commands::init_prj::AdoptionBlockStatus::StaleVersion { version } => {
                findings.push(format!(
                    "project-adoption: stale block version v={} at '{}' — re-run ce-ai init-prj --tier {} to upgrade",
                    version,
                    p.path.display(),
                    p.tier.as_str()
                ));
            }
            crate::commands::init_prj::AdoptionBlockStatus::DriftDetected
            | crate::commands::init_prj::AdoptionBlockStatus::MalformedBlock
            | crate::commands::init_prj::AdoptionBlockStatus::BlockMissing
            | crate::commands::init_prj::AdoptionBlockStatus::ReadError => {
                findings.push(format!(
                    "project-adoption: block SHA drift detected at '{}'",
                    p.path.display()
                ));
            }
        }
    }

    // Git Hooks & Worktree Health Probes
    if let Ok(repo_root) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if repo_root.status.success() {
            let root_str = String::from_utf8_lossy(&repo_root.stdout)
                .trim()
                .to_string();
            let root_path = std::path::Path::new(&root_str);

            let codegraph_dir = root_path.join(".codegraph");
            if !codegraph_dir.exists() {
                println!("doctor-info: codegraph index (.codegraph/) not initialized");
            }

            if let Ok(hooks_output) = std::process::Command::new("git")
                .args(["config", "--get", "core.hooksPath"])
                .current_dir(root_path)
                .output()
            {
                if hooks_output.status.success() {
                    let raw_val = String::from_utf8_lossy(&hooks_output.stdout);
                    let hooks_val = raw_val.trim().trim_end_matches('/').trim_end_matches('\\');
                    let hooks_path = std::path::Path::new(hooks_val);
                    if !hooks_val.ends_with(".githooks")
                        && hooks_path.file_name() != Some(std::ffi::OsStr::new(".githooks"))
                    {
                        findings.push(format!(
                            "git-hooks: core.hooksPath set to '{}', expected '.githooks'",
                            hooks_val
                        ));
                    } else {
                        let pre_commit = root_path.join(".githooks").join("pre-commit");
                        if !pre_commit.exists() {
                            findings.push("git-hooks: .githooks/pre-commit missing".into());
                        }
                    }
                } else {
                    println!("doctor-info: git-hooks core.hooksPath not set");
                }
            }

            if let Ok(wt_output) = std::process::Command::new("git")
                .args(["worktree", "list", "--porcelain"])
                .current_dir(root_path)
                .output()
            {
                if wt_output.status.success() {
                    let canonical_root = root_path
                        .canonicalize()
                        .unwrap_or_else(|_| root_path.to_path_buf());
                    let stdout = String::from_utf8_lossy(&wt_output.stdout);
                    for line in stdout.lines() {
                        if let Some(path_str) = line.strip_prefix("worktree ") {
                            let wt_path = std::path::Path::new(path_str);
                            let canonical_wt = wt_path
                                .canonicalize()
                                .unwrap_or_else(|_| wt_path.to_path_buf());
                            if canonical_wt != canonical_root {
                                println!(
                                    "doctor-info: active sibling worktree detected at '{}'",
                                    path_str
                                );
                            }
                        }
                    }
                }
            }

            // Branch Protection Health Probe (context-resilience R1):
            // verifies the GitHub platform boundary requires status checks
            // on `main`. Non-GitHub remotes and an unavailable `gh` degrade
            // to notices — only a *verifiably* unprotected main is a finding.
            if let Some(slug) = github_slug_from_remote(root_path) {
                match std::process::Command::new("gh")
                    .args(["api", &format!("repos/{slug}/branches/main/protection")])
                    .output()
                {
                    Ok(out) if out.status.success() => {
                        let body = String::from_utf8_lossy(&out.stdout);
                        if !body.contains("required_status_checks") {
                            findings.push(
                                "branch-protection: main missing required status checks".into(),
                            );
                        } else if !body.contains("required_pull_request_reviews") {
                            println!(
                                "doctor-info: branch-protection: PR reviews not required on main (single-developer flow)"
                            );
                        }
                    }
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if stderr.contains("404") || stderr.contains("Not Found") {
                            findings
                                .push("branch-protection: missing or unconfigured for main".into());
                        } else if !ctx.quiet {
                            println!(
                                "doctor-info: cannot verify branch protection ({})",
                                stderr.trim()
                            );
                        }
                    }
                    Err(err) if !ctx.quiet => {
                        println!("doctor-info: cannot verify branch protection (gh: {err})");
                    }
                    Err(_) => {}
                }
            }
        }
    }

    // Incomplete-operation journal (#166): diagnosis before auto-recovery.
    if let Some(cmd_name) = crate::state::journal::recorded_command(&ctx.config_dir) {
        findings.push(format!(
            "install-journal: incomplete '{cmd_name}' operation detected — the next install/sync rolls it back automatically"
        ));
    }

    for finding in &findings {
        println!("{finding}");
    }
    if findings.is_empty() {
        println!("doctor: ok");
        return Ok(());
    }
    Err(CeError::Runtime(format!(
        "doctor found {} finding(s)",
        findings.len()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_doctor_strict_flag_default() {
        let args = Args::default();
        assert!(!args.strict);
    }

    #[test]
    fn test_doctor_runs_on_clean_context() {
        let tmp = TempDir::new().unwrap();
        let ctx = Context {
            config_dir: tmp.path().to_path_buf(),
            opencode_config_dir: tmp.path().to_path_buf(),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: true,
        };
        std::fs::write(
            ctx.config_dir.join("skills-registry.json"),
            r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
        )
        .unwrap();
        let args = Args::default();
        assert!(run(&ctx, &args).is_ok());
    }
}

#[cfg(test)]
mod branch_protection_tests {
    use super::github_slug_from_url;

    #[test]
    fn github_slug_parses_ssh_https_and_rejects_other_hosts() {
        assert_eq!(
            github_slug_from_url("git@github.com:mastepanoski/ce-ai.git").as_deref(),
            Some("mastepanoski/ce-ai")
        );
        assert_eq!(
            github_slug_from_url("https://github.com/mastepanoski/ce-ai/").as_deref(),
            Some("mastepanoski/ce-ai")
        );
        assert_eq!(
            github_slug_from_url("https://gitlab.com/group/proj.git"),
            None
        );
    }
}
