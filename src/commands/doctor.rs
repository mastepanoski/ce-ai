//! `ce-ai doctor`: report config-validity, diff (drift), and state-consistency
//! findings; exits non-zero when any finding exists.

use std::collections::BTreeMap;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::read_config;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::diff::{self, Action};
use crate::state::state::State;

pub fn run(ctx: &Context) -> Result<(), CeError> {
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
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    let has_entry = state
        .installed_harnesses
        .iter()
        .any(|h| h["name"].as_str() == Some("opencode"));
    if has_entry != manifest.is_ok() {
        findings
            .push("state-inconsistent: opencode state entry and install manifest disagree".into());
    }

    // Model Assignment Drift Probe
    if opencode_json.exists() {
        if let Ok(config_json) = read_config(&opencode_json) {
            if let Some(agents) = config_json.get("agent").and_then(|a| a.as_object()) {
                for (slot, val) in agents {
                    if let Some(model_str) = val.get("model").and_then(|m| m.as_str()) {
                        if !model_str.is_empty() {
                            let state_model = state
                                .model_assignments
                                .get(slot)
                                .map(|a| format!("{}/{}", a.provider_id, a.model_id));
                            if state_model.as_deref() != Some(model_str) {
                                findings.push(format!(
                                    "model-assignment-drift: slot '{slot}' configured as '{model_str}' in opencode.json but unrecorded or mismatched in state.json (run 'ce-ai sync' to reconcile)"
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Project adoption health checks
    for p in &state.projects {
        let agents_file = p.path.join(&p.file);
        if !agents_file.exists() {
            findings.push(format!(
                "project-adoption: missing instruction file '{}' at '{}'",
                p.file,
                p.path.display()
            ));
        } else if let Ok(text) = std::fs::read_to_string(&agents_file) {
            let inner_body = crate::commands::init_prj::render_block_content(p.tier);
            let expected_sha = crate::commands::init_prj::compute_sha256(inner_body);
            if !text.contains(&expected_sha) {
                findings.push(format!(
                    "project-adoption: block SHA drift detected at '{}'",
                    p.path.display()
                ));
            }
        }
    }

    // Companion tool health checks.
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let engram_db = std::path::Path::new(&home)
            .join(".engram")
            .join("engram.db");
        if !engram_db.exists() {
            println!("doctor-info: engram db (~/.engram/engram.db) not found");
        }
    }

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

            // Git Hooks Health Probe
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

            // GitHub Branch Protection Health Probe
            let remote_url = std::process::Command::new("git")
                .args(["remote", "get-url", "origin"])
                .current_dir(root_path)
                .output();

            let is_github = remote_url
                .map(|o| {
                    o.status.success() && String::from_utf8_lossy(&o.stdout).contains("github.com")
                })
                .unwrap_or(false);

            let gh_authenticated = std::process::Command::new("gh")
                .args(["auth", "status"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if is_github && gh_authenticated {
                let repo_name = std::process::Command::new("gh")
                    .args([
                        "repo",
                        "view",
                        "--json",
                        "nameWithOwner",
                        "-q",
                        ".nameWithOwner",
                    ])
                    .current_dir(root_path)
                    .output();

                if let Ok(repo_out) = repo_name {
                    if repo_out.status.success() {
                        let repo_str = String::from_utf8_lossy(&repo_out.stdout).trim().to_string();
                        let prot_check = std::process::Command::new("gh")
                            .args([
                                "api",
                                &format!("repos/{}/branches/main/protection", repo_str),
                            ])
                            .output();

                        if let Ok(prot_out) = prot_check {
                            if !prot_out.status.success() {
                                let stderr = String::from_utf8_lossy(&prot_out.stderr);
                                if stderr.contains("403") {
                                    println!("doctor-info: token lacks admin permissions for branch protection API on {}", repo_str);
                                } else {
                                    findings.push(format!("branch-protection: main branch protection missing or unconfigured on {}", repo_str));
                                }
                            }
                        }
                    }
                }
            } else if is_github {
                println!("doctor-info: gh CLI unauthenticated or offline, skipping branch protection probe");
            }

            // Git Sibling Worktree Probe
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
        }
    }

    let which_tool = if cfg!(windows) { "where" } else { "which" };
    let rtk_on_path = std::process::Command::new(which_tool)
        .arg("rtk")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !rtk_on_path {
        println!("doctor-info: rtk binary not found on PATH");
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
