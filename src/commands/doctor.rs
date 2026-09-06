//! `ce-ai doctor`: report config-validity, diff (drift), state-consistency,
//! companion tool readiness, version freshness, and skill health findings.
//! Exits non-zero when any finding exists.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::opencode::config::read_config;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::source::tools_registry::{
    detect_tool_freshness, is_skill_configured, FreshnessStatus, ToolsRegistryCache,
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
    let state = State::load_with_workspace_overrides(
        &ctx.config_dir.join("state.json"),
        ctx.workspace_root.as_deref(),
    )?;
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    let mut findings: Vec<String> = Vec::new();

    // Config validity: opencode.json must parse (D4).
    let opencode_json = opencode_dir.join("opencode.json");
    if let Err(err) = read_config(&opencode_json) {
        findings.push(format!("config-invalid: {err}"));
    }

    // Diff: managed files vs the install manifest (SU-3).
    let manifest = InstallManifest::load(&opencode_dir);
    if let Ok(manifest) = &manifest {
        let desired: BTreeMap<String, String> = manifest
            .files
            .iter()
            .map(|f| (f.path.clone(), f.sha256.clone()))
            .collect();
        let managed = opencode_dir.join(MANAGED_DIR);
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
    let repo_root = ctx.repo_root();
    let has_entry = state.installed_harnesses.iter().any(|h| {
        if h["name"].as_str() != Some("opencode") {
            return false;
        }
        match h["scope"].as_str() {
            Some("workspace") => h["target_dir"]
                .as_str()
                .map(|d| std::path::Path::new(d) == repo_root.as_path())
                .unwrap_or(true),
            Some("global") | None => true,
            _ => false,
        }
    });
    if has_entry != manifest.is_ok() {
        findings
            .push("state-inconsistent: opencode state entry and install manifest disagree".into());
    }

    if has_entry && !crate::opencode::plugins::has_session_start_plugin(&opencode_dir) {
        findings.push(format!(
            "opencode: SessionStart plugin missing or outdated in '{}' — run 'ce-ai sync' or 'ce-ai install --harness opencode' to update",
            opencode_dir.display()
        ));
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

    // Companion Tools Readiness & Version Freshness Probe (#112, #293)
    let registry = ToolsRegistryCache::load_or_default(ctx);
    for (name, info) in &registry.tools {
        let freshness = detect_tool_freshness(ctx, name, info);

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

    // RTK Hook Readiness & Limitation Disclosure Probe (#308)
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| ctx.config_dir.clone());
    let rtk_available = crate::harness::rtk::is_rtk_available();

    if rtk_available {
        println!(
            "doctor-info: rtk output compression: active (note: rtk command filters may alter or swallow stdout on wrapped commands like 'gh issue view --comments'; opt out via --skip-rtk or CE_AI_SKIP_RTK=1 if needed)"
        );
        for h in &state.installed_harnesses {
            if let Some(name) = h["name"].as_str() {
                if let Ok(kind) = name.parse::<HarnessKind>() {
                    if crate::harness::rtk::is_rtk_supported(kind)
                        && !crate::harness::rtk::is_rtk_hook_configured(&home_dir, kind)
                    {
                        let msg = format!(
                            "rtk-hook-missing: hook not configured for installed supported harness '{kind}' (run 'ce-ai install --harness {kind}' or 'rtk init -g')"
                        );
                        if args.strict {
                            findings.push(msg);
                        } else {
                            println!("doctor-warn: {}", msg);
                        }
                    }
                }
            }
        }
    } else {
        let has_supported_installed = state.installed_harnesses.iter().any(|h| {
            h["name"]
                .as_str()
                .and_then(|n| n.parse::<HarnessKind>().ok())
                .map(crate::harness::rtk::is_rtk_supported)
                .unwrap_or(false)
        });
        if has_supported_installed {
            let msg = "rtk-missing: companion tool 'rtk' not installed on PATH for supported harness(es) (suggested: 'ce-ai tools install rtk')".to_string();
            if args.strict {
                findings.push(msg);
            } else {
                println!("doctor-warn: {}", msg);
            }
        }
    }

    // Skill Suggestions Probe (#112, #293)
    for (name, skill) in &registry.skills {
        if !is_skill_configured(ctx, name) {
            println!(
                "doctor-info: skill-suggestion: {} (run '{}')",
                name, skill.resolve_cmd
            );
        }
    }

    // GitHub token info for install/upgrade
    if crate::source::release::resolve_github_token().is_some() {
        println!("doctor-info: github-token present (authenticated API quota)");
    } else {
        println!(
            "doctor-info: github-token not set (unauthenticated mode with resilient web fallback)"
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

        let claude_dir = p.path.join(".claude");
        if claude_dir.exists() {
            let settings = claude_dir.join("settings.json");
            if !crate::harness::claude::has_session_start_hook(&settings) {
                findings.push(format!(
                    "project-adoption: Claude Code SessionStart hook missing at '{}' — re-run ce-ai init-prj --tier {} to configure",
                    settings.display(),
                    p.tier.as_str()
                ));
            }
        }

        let github_dir = p.path.join(".github");
        if github_dir.exists() {
            let hooks_file = github_dir.join("hooks").join("hooks.json");
            if !crate::harness::copilot::has_session_start_hook(&hooks_file) {
                findings.push(format!(
                    "project-adoption: Copilot CLI sessionStart hook missing at '{}' — re-run ce-ai init-prj --tier {} to configure",
                    hooks_file.display(),
                    p.tier.as_str()
                ));
            }
        }

        let codex_dir = p.path.join(".codex");
        if codex_dir.exists() {
            let config_file = codex_dir.join("config.toml");
            if !crate::harness::codex::has_session_start_hook(&config_file) {
                findings.push(format!(
                    "project-adoption: Codex CLI SessionStart hook missing at '{}' — re-run ce-ai init-prj --tier {} to configure",
                    config_file.display(),
                    p.tier.as_str()
                ));
            }
        }

        let pi_dir = p.path.join(".pi");
        if pi_dir.exists() {
            let extension_file = pi_dir
                .join("extensions")
                .join(crate::harness::pi::PI_EXTENSION_FILENAME);
            if !crate::harness::pi::has_session_start_hook(&extension_file) {
                findings.push(format!(
                    "project-adoption: Pi before_agent_start extension missing at '{}' — re-run ce-ai init-prj --tier {} to configure",
                    extension_file.display(),
                    p.tier.as_str()
                ));
            }
        }

        let cursor_dir = p.path.join(".cursor");
        if cursor_dir.exists() {
            let hooks_file = cursor_dir.join("hooks.json");
            if !crate::harness::cursor::has_session_start_hook(&hooks_file) {
                findings.push(format!(
                    "project-adoption: Cursor sessionStart hook missing at '{}' — re-run ce-ai init-prj --tier {} to configure",
                    hooks_file.display(),
                    p.tier.as_str()
                ));
            }
        }

        let agents_dir = p.path.join(".agents");
        if agents_dir.exists() {
            let hooks_file = agents_dir.join("hooks.json");
            if !crate::harness::agy::has_pre_invocation_hook(&hooks_file) {
                findings.push(format!(
                    "project-adoption: Antigravity PreInvocation hook missing at '{}' — re-run ce-ai init-prj --tier {} to configure",
                    hooks_file.display(),
                    p.tier.as_str()
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
                println!("doctor-info: codegraph index (.codegraph/) not initialized (suggested: 'ce-ai tools init codegraph')");
            }

            let githooks_dir = root_path.join(".githooks");
            let uses_githooks_convention = githooks_dir.exists();

            if let Ok(hooks_output) = std::process::Command::new("git")
                .args(["config", "--get", "core.hooksPath"])
                .current_dir(root_path)
                .output()
            {
                if hooks_output.status.success() {
                    let raw_val = String::from_utf8_lossy(&hooks_output.stdout);
                    let hooks_val = raw_val.trim().trim_end_matches('/').trim_end_matches('\\');
                    let hooks_path = std::path::Path::new(hooks_val);
                    let points_to_githooks = hooks_val.ends_with(".githooks")
                        || hooks_path.file_name() == Some(std::ffi::OsStr::new(".githooks"));
                    if points_to_githooks {
                        let pre_commit = root_path.join(".githooks").join("pre-commit");
                        if !pre_commit.exists() {
                            findings.push("git-hooks: .githooks/pre-commit missing".into());
                        }
                    } else if uses_githooks_convention {
                        // Project has already adopted the .githooks convention (the
                        // directory exists), so a hooksPath pointing elsewhere is drift,
                        // not an unrelated hooks manager.
                        findings.push(format!(
                            "git-hooks: core.hooksPath set to '{}', expected '.githooks'",
                            hooks_val
                        ));
                    } else {
                        println!(
                            "doctor-info: git-hooks core.hooksPath set to '{}' (not the .githooks convention; skipping)",
                            hooks_val
                        );
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

    // Pedagogical Guardrail status info (Issue #114)
    if let Ok(state) = State::load(&ctx.state_path()) {
        if let Some(guard) = &state.guardrail {
            if guard.enabled {
                println!(
                    "doctor-info: pedagogical guardrail enabled (level: {}, scope: {})",
                    guard.level,
                    guard.harness.as_deref().unwrap_or("global")
                );
            }
        }
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
#[path = "tests/doctor.rs"]
mod tests;
