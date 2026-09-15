//! `ce-ai doctor`: report config-validity, diff (drift), state-consistency,
//! companion tool readiness, version freshness, and skill health findings.
//! Exits non-zero when any finding exists.

use std::collections::BTreeMap;

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

/// Diff findings for non-OpenCode managed harnesses (claude, kimi) when their
/// manifests exist — same engine as the OpenCode diff, per-harness managed
/// trees, harness-prefixed finding strings (`diff: <harness> <kind> <path>`).
pub(crate) fn non_opencode_diff_findings(home_dir: &std::path::Path) -> Vec<String> {
    let mut findings = Vec::new();
    for kind in crate::commands::workflow::DRIFT_PROBE_HARNESSES {
        let harness_dir = kind.harness_dir(home_dir);
        if let Ok(manifest) = InstallManifest::load(&harness_dir) {
            let desired: BTreeMap<String, String> = manifest
                .files
                .iter()
                .map(|f| (f.path.clone(), f.sha256.clone()))
                .collect();
            let managed = harness_dir.join(MANAGED_DIR);
            for action in diff::diff(&desired, &desired, &managed).actions {
                let (verb, path) = match action {
                    Action::Copy { path } => ("missing", path),
                    Action::Restore { path } => ("modified", path),
                    Action::Remove { path } => ("stale", path),
                };
                findings.push(format!("diff: {kind} {verb} {path}"));
            }
        }
    }
    findings
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

    // Diff for non-OpenCode managed harnesses (claude, kimi) when their
    // manifests exist — same engine, per-harness managed trees.
    let home_dir_for_diff = crate::harness::home_dir_from_ctx(ctx);
    findings.extend(non_opencode_diff_findings(&home_dir_for_diff));

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
        if let Some(note) =
            crate::commands::models::check_code_review_mid_tier_note(&state, &config)
        {
            println!("doctor-info: {note}");
        }
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
    let home_dir = crate::harness::home_dir_from_ctx(ctx);
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

    // Claude Code Native Plugin Marketplace Divergence Probe (#327)
    let claude_dir = HarnessKind::Claude.harness_dir(&home_dir);
    let cwd = std::env::current_dir().unwrap_or_else(|_| repo_root.clone());
    let claude_divergences =
        crate::harness::claude::check_claude_marketplace_divergence(&state, &cwd, &claude_dir);
    for d in claude_divergences {
        let marketplace = d
            .plugin_id
            .split_once('@')
            .map(|(_, m)| m)
            .unwrap_or("compound-engineering-plugin");
        let update_cmd = format!(
            "claude plugin marketplace update {marketplace} && claude plugin update {}",
            d.plugin_id
        );
        let norm_native = crate::harness::claude::normalize_plugin_version(&d.native_version);
        let norm_ce = crate::harness::claude::normalize_plugin_version(&d.ce_version);
        println!(
            "doctor-info: claude native plugin marketplace divergence detected for '{}' (scope: {}): native marketplace has v{} but ce-ai managed harness is v{} (run '{}' to update)",
            d.plugin_id, d.scope, norm_native, norm_ce, update_cmd
        );
    }

    // Kimi Code Native Plugin Manager Divergence Probe:
    // Kimi's native plugin manager (~/.kimi-code/plugins/installed.json) can
    // shadow the ce-ai managed tree with its own pinned version.
    let kimi_dir = HarnessKind::Kimi.harness_dir(&home_dir);
    let kimi_divergences =
        crate::harness::kimi::check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    for d in &kimi_divergences {
        let norm_native = crate::harness::claude::normalize_plugin_version(&d.native_version);
        let norm_ce = crate::harness::claude::normalize_plugin_version(&d.ce_version);
        println!(
            "doctor-info: kimi native plugin divergence detected for '{}' : native plugin manager has v{} but ce-ai managed harness is v{} (update the plugin via Kimi's native plugin manager, or uninstall the native plugin to use the ce-ai managed tree exclusively)",
            d.plugin_id, norm_native, norm_ce
        );
    }
    if let Some(orphan) = crate::harness::kimi::check_kimi_orphan_managed_tree(&kimi_dir) {
        println!(
            "doctor-warn: kimi managed tree '{}' is not referenced by Kimi config (extra_skill_dirs) — the ce-ai managed skills are inactive for Kimi (reference the tree from ~/.kimi-code/config.toml, or remove it with 'ce-ai uninstall --harness kimi')",
            orphan.managed_dir.display()
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

    // OpenSpec Tasks Desync & Worktree/Resolution Health Probes (Issues #313, #337):
    let repo_root = ctx.repo_root();
    let branch = crate::commands::workflow::probe_git_branch(&repo_root);
    let current_wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
    let openspec_info =
        crate::commands::workflow::probe_openspec_context_in(&repo_root, &current_wf);

    // Mtime fallback warning (Issue #337 Gap 1)
    let is_mtime_fallback = current_wf
        .as_ref()
        .and_then(|w| w.resolution)
        .map(|r| r == crate::state::state::FeatureResolution::MtimeFallback)
        .unwrap_or(false)
        || openspec_info
            .as_ref()
            .and_then(|i| i.resolution)
            .map(|r| r == crate::state::state::FeatureResolution::MtimeFallback)
            .unwrap_or(false);
    if is_mtime_fallback {
        let feat = openspec_info
            .as_ref()
            .map(|i| i.feature.as_str())
            .or_else(|| current_wf.as_ref().and_then(|w| w.feature_name.as_deref()))
            .unwrap_or("unknown");
        println!(
            "doctor-warn: workflow feature '{feat}' resolved via mtime fallback (unreliable without git branch)"
        );
    }

    // Uncommitted OpenSpec spec warning (Issue #337 Gap 2)
    if let Some(ref info) = openspec_info {
        if info.is_uncommitted {
            println!(
                "doctor-warn: openspec change '{}': esta spec no está commiteada — puede no ser visible en otros worktrees",
                info.feature
            );
        }

        let touched_files = crate::commands::workflow::probe_feature_touched_files(&repo_root);
        if let Some(desync) = crate::commands::workflow::reconcile_tasks_with_git(
            &repo_root,
            &info.feature,
            &info.path.join("tasks.md"),
            &touched_files,
        ) {
            let count = if desync.desynced_tasks.is_empty() {
                desync.total_tasks
            } else {
                desync.desynced_tasks.len()
            };
            println!(
                "doctor-warn: openspec tasks desync in '{}': {count} unchecked task(s) with git modifications (progress: {}/{})",
                info.feature, desync.completed_tasks, desync.total_tasks
            );
        }
    }

    // OpenSpec Unarchived Completed Changes Probe (Issue #323):
    // Detects fully-completed OpenSpec changes lingering outside openspec/changes/archive/.
    let unarchived = crate::commands::workflow::probe_unarchived_completed_changes(&repo_root);
    for item in &unarchived {
        println!(
            "doctor-warn: openspec change '{}' is complete ({}/{} tasks) but not archived — see openspec/changes/archive/README.md",
            item.feature, item.completed_tasks, item.total_tasks
        );
    }

    // Documentation Technical Debt Diagnostic Engine & Probes (doc-debt-engine-and-probes)
    let doc_hygiene_cfg = state.doc_hygiene();
    let doc_debt =
        crate::commands::workflow::probe_doc_debt(&repo_root, branch.as_deref(), &doc_hygiene_cfg);

    if let crate::commands::workflow::ProbeStatus::Debt(desync_findings) = &doc_debt.openspec_desync
    {
        for f in desync_findings {
            let reason_msg = match f.reason {
                crate::commands::workflow::DesyncReason::ParentTasksCompleteSubtasksOpen => {
                    "is complete with open subtasks"
                }
                crate::commands::workflow::DesyncReason::CodeMergedToMain => {
                    "has code merged to main"
                }
                crate::commands::workflow::DesyncReason::ReleaseVersionSurpassed(_) => {
                    "cited release version is surpassed in Cargo.toml"
                }
            };
            println!(
                "doctor-warn: openspec change '{}' {} (progress: {}/{}) — run 'ce-ai archive {} --auto-mark' or 'ce-ai archive {} --status \"...\"'",
                f.feature, reason_msg, f.completed_tasks, f.total_tasks, f.feature, f.feature
            );
        }
    }

    if let crate::commands::workflow::ProbeStatus::Debt(stale_findings) = &doc_debt.stale_pending {
        for f in stale_findings {
            let git_msg = match f.source {
                crate::commands::workflow::InactivitySource::GitCommitDate => "with no git commits",
                crate::commands::workflow::InactivitySource::FilesystemMtime => "with no activity",
            };
            println!(
                "doctor-warn: openspec change '{}' has been pending for {} days {} (progress: {}/{}) — resume, shelve, or archive with '--status \"superseded\"'",
                f.feature, f.days_inactive, git_msg, f.completed_tasks, f.total_tasks
            );
        }
    }

    if let crate::commands::workflow::ProbeStatus::Debt(drift_findings) = &doc_debt.solution_drift {
        for f in drift_findings {
            let display_path = f
                .solution_path
                .strip_prefix("docs/solutions/")
                .unwrap_or(&f.solution_path);
            for dead_path in &f.dead_paths {
                println!(
                    "doctor-warn: solution '{}' references non-existent path '{}'",
                    display_path, dead_path
                );
            }
            for field in &f.missing_frontmatter_fields {
                println!(
                    "doctor-warn: solution '{}' missing required YAML frontmatter: {}",
                    display_path, field
                );
            }
        }
    }

    if let crate::commands::workflow::ProbeStatus::Debt(compact_finding) =
        &doc_debt.archive_compaction
    {
        println!(
            "doctor-warn: archive has {} uncompacted packages (>{} threshold); run 'ce-ai archive compact' to roll up aged changes into milestone summaries",
            compact_finding.uncompacted_count, compact_finding.threshold
        );
    }

    // Living System Specifications Health Probe (living-system-specs-promotion)
    let spec_warnings = crate::commands::spec::probe_specs_health(&repo_root);
    for warn in &spec_warnings {
        println!("doctor-warn: {warn}");
    }

    // Observe-only Ship-readiness probe (Issue #354): Stage 6 + code-review gaps.
    // Non-fatal: never added to `findings`, exit code unaffected. Only runs for
    // adopted workspaces to avoid noise on unrelated repositories.
    let commits_ahead = crate::commands::workflow::probe_commits_ahead(&repo_root);
    if commits_ahead > 0 && state.is_project_adopted(&repo_root) {
        let (_, dirty) = crate::commands::workflow::probe_git_dirty_files(&repo_root);
        let stage6 = crate::commands::workflow::probe_stage6_artifact(&repo_root, &dirty);
        let head_sha = crate::commands::workflow::probe_git_head_full_sha(&repo_root);
        let receipt = state.review_receipt_for_branch(&repo_root, branch.as_deref());
        let stage = current_wf.as_ref().map(|w| w.stage).unwrap_or_default();
        for gap in crate::commands::workflow::evaluate_ship_readiness(
            commits_ahead,
            stage6,
            receipt,
            head_sha.as_deref(),
            stage,
        ) {
            println!("doctor-warn: ship-readiness: {}", gap.describe());
        }
    }

    // Gate Check Telemetry Metrics (Issue #333, #334)
    let gate_stats = crate::commands::gate::load_gate_stats(&ctx.config_dir).unwrap_or_default();
    if gate_stats.total_observed > 0 {
        let edge_total = gate_stats.mtime_fallback
            + gate_stats.worktree_uncommitted
            + gate_stats.stale_cycle_guard;
        let blocked_part = if gate_stats.blocked > 0 {
            format!("{} blocked, ", gate_stats.blocked)
        } else {
            String::new()
        };
        println!(
            "gate-check: {} observed ({}{} would-block, {} pass, {} undetermined, {} edge-case: {} mtime_fallback, {} worktree_uncommitted, {} stale_cycle_guard)",
            gate_stats.total_observed,
            blocked_part,
            gate_stats.would_block,
            gate_stats.pass,
            gate_stats.undetermined,
            edge_total,
            gate_stats.mtime_fallback,
            gate_stats.worktree_uncommitted,
            gate_stats.stale_cycle_guard,
        );
    }

    // Gate Check Blocked Receipts Advisory (Issue #334)
    if let Some(wf) = current_wf.as_ref() {
        if let Some(ref feat) = wf.feature_name {
            if let Some(receipt) = state.gate_receipts.get(feat) {
                if receipt.decision == crate::state::state::GateDecision::Blocked {
                    println!(
                        "doctor-warn: gate-check: write blocked on '{}' for feature '{}' (missing: {})",
                        receipt.target_path,
                        receipt.feature,
                        receipt.missing_artifacts.join(", ")
                    );
                }
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
