//! `ce-ai status`: print installed harnesses, versions, and drift (CC-1).

use std::collections::BTreeMap;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::diff::{self, Action};
use crate::state::state::State;

use crate::harness::HarnessKind;
use std::collections::HashSet;

pub fn run(ctx: &Context) -> Result<(), CeError> {
    let state = State::load_with_workspace_overrides(
        &ctx.config_dir.join("state.json"),
        ctx.workspace_root.as_deref(),
    )?;
    let mut installed_list = Vec::new();
    let mut seen = HashSet::new();

    for harness in &state.installed_harnesses {
        let name = harness["name"].as_str().unwrap_or("?").to_string();
        let ver = harness["version"].as_str().unwrap_or("?").to_string();
        let src = harness["source"]["kind"]
            .as_str()
            .unwrap_or("?")
            .to_string();
        seen.insert(name.clone());
        installed_list.push((name, ver, src));
    }

    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::Path::new(&home);
        for h in HarnessKind::detect_ce_installed_harnesses(home_path) {
            let name = h.to_string();
            if !seen.contains(&name) {
                seen.insert(name.clone());
                installed_list.push((name, "host-detected".to_string(), "local".to_string()));
            }
        }
    }

    if installed_list.is_empty() {
        println!("installed: none");
    } else {
        let has_local_source = installed_list
            .iter()
            .any(|(_, ver, src)| ver == "local" || src == "local");
        for (name, version, source) in &installed_list {
            println!("installed: {name} ({version}, source: {source})");
            if ctx.verbose {
                println!("  verbose: enabled for {name}");
            }
        }
        if has_local_source {
            let latest_tag = state.latest_release_tag.as_deref().unwrap_or("latest");
            println!("upstream: latest GitHub release available is {latest_tag}");
            println!("recommendation: Run 'ce-ai upgrade' to update from local source to latest release.");
        }
    }

    // Adoption states (canonical-skills-adoption R19).
    if !state.skill_surfaces.is_empty() {
        for surface in &state.skill_surfaces {
            match surface.status.as_str() {
                "adopted" => {
                    println!(
                        "skills: {} adopted ({})",
                        surface.harness,
                        surface.root.display()
                    );
                }
                "declined" => {
                    println!(
                        "skills: {} declined ({}) — run 'ce-ai skills adopt' to reconsider",
                        surface.harness,
                        surface.root.display()
                    );
                }
                "orphaned" => {
                    println!(
                        "skills: {} orphaned ({}) — run 'ce-ai skills adopt' to re-adopt",
                        surface.harness,
                        surface.root.display()
                    );
                }
                _ => {}
            }
        }
    }

    // Drift: compare managed files on disk against the install manifest (SU-3).
    // OpenCode keeps its historical output format; claude/kimi manifests
    // (when present) report with an explicit harness prefix.
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    let mut any_manifest = false;
    let mut total_actions = 0usize;
    let mut print_drift = |label: Option<&str>, config_dir: &std::path::Path| {
        let Ok(manifest) = InstallManifest::load(config_dir) else {
            return;
        };
        any_manifest = true;
        let desired: BTreeMap<String, String> = manifest
            .files
            .iter()
            .map(|f| (f.path.clone(), f.sha256.clone()))
            .collect();
        let managed = config_dir.join(MANAGED_DIR);
        let drift = diff::diff(&desired, &desired, &managed);
        total_actions += drift.actions.len();
        for action in &drift.actions {
            let (kind, path) = match action {
                Action::Copy { path } => ("missing", path),
                Action::Restore { path } => ("modified", path),
                Action::Remove { path } => ("stale", path),
            };
            match label {
                Some(harness) => println!("drift: {harness}: {kind} {path}"),
                None => println!("drift: {kind} {path}"),
            }
        }
    };
    print_drift(None, &opencode_dir);
    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::Path::new(&home);
        for kind in crate::commands::workflow::DRIFT_PROBE_HARNESSES {
            print_drift(Some(kind.as_str()), &kind.harness_dir(home_path));
        }
    }
    if !any_manifest {
        println!("drift: unknown (no install manifest)");
    } else if total_actions == 0 {
        println!("drift: none");
    }

    // Project Adoption Status
    if !state.projects.is_empty() {
        println!("projects: {} adopted", state.projects.len());
        for p in &state.projects {
            let agents_file = p.path.join(&p.file);
            let status_str = match crate::commands::init_prj::check_adoption_block_status(
                &agents_file,
                p.tier,
            ) {
                crate::commands::init_prj::AdoptionBlockStatus::Ok => "OK".to_string(),
                crate::commands::init_prj::AdoptionBlockStatus::StaleVersion { version } => {
                    format!(
                        "STALE BLOCK v={} — re-run ce-ai init-prj --tier {} to upgrade",
                        version,
                        p.tier.as_str()
                    )
                }
                crate::commands::init_prj::AdoptionBlockStatus::DriftDetected => {
                    "DRIFT DETECTED".to_string()
                }
                crate::commands::init_prj::AdoptionBlockStatus::MalformedBlock => {
                    "MALFORMED BLOCK".to_string()
                }
                crate::commands::init_prj::AdoptionBlockStatus::BlockMissing => {
                    "BLOCK MISSING".to_string()
                }
                crate::commands::init_prj::AdoptionBlockStatus::FileMissing => {
                    "MISSING".to_string()
                }
                crate::commands::init_prj::AdoptionBlockStatus::ReadError => {
                    "READ ERROR".to_string()
                }
            };
            println!(
                "  - {} (tier: {:?}, file: {}, status: {})",
                p.path.display(),
                p.tier,
                p.file,
                status_str
            );
        }
    } else {
        println!("projects: none adopted");
    }

    // Pedagogical Guardrail Status (Issue #114)
    if let Some(guard) = &state.guardrail {
        if guard.enabled {
            println!(
                "guardrail: enabled (level: {}, scope: {})",
                guard.level,
                guard.harness.as_deref().unwrap_or("global")
            );
        } else {
            println!("guardrail: disabled");
        }
    }

    // Git Environment State
    let root = ctx.repo_root();
    let branch = crate::commands::workflow::probe_git_branch(&root);
    if let Some(ref b) = branch {
        let (clean, dirty_files) = crate::commands::workflow::probe_git_dirty_files(&root);
        if clean {
            println!("git: {b} (clean)");
        } else {
            println!("git: {b} ({} modified files)", dirty_files.len());
        }
    }

    // Active Workflow State & Mtime Fallback Warning (Issue #337)
    let state_path = ctx.config_dir.join("state.json");
    let _ = crate::commands::workflow::maybe_auto_checkpoint(ctx, &root, &state_path);
    let effective_state = State::load(&state_path).unwrap_or_else(|_| state.clone());
    let current_wf = effective_state.current_workflow_for_branch(&root, branch.as_deref());
    let openspec_info = crate::commands::workflow::probe_openspec_context_in(&root, &current_wf);

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
            "workflow-warn: active feature '{feat}' resolved via mtime fallback (unreliable without git branch)"
        );
    }

    Ok(())
}
