//! `ce-ai sync`: reconcile the desired source tree against installed files
//! (SU-1..SU-4). Desired manifest comes from the current source; the diff
//! engine plans copy/restore/remove actions that `--dry-run` only prints.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::custom::{
    plugin_rel as custom_plugin_rel, skill_rel as custom_skill_rel, CustomHarnessConfig,
};
use crate::harness::registration::registration_spec;
use crate::harness::HarnessKind;
use crate::opencode::manifest::{InstallManifest, ManifestFile};
use crate::opencode::plugins::MANAGED_DIR;
use crate::source::cache::managed_tree;
use crate::state::diff::{self, sha256_hex, Action};
use crate::state::state::State;
use crate::state::write_atomic;

#[derive(clap::Args, Debug, Default)]
pub struct Args {
    /// Watch managed configuration paths and continuously re-sync upon drift.
    #[arg(long)]
    pub watch: bool,
    /// Polling interval in milliseconds (default: 2000).
    #[arg(long)]
    pub interval_ms: Option<u64>,
    /// Maximum polling passes before exit (used in integration tests).
    #[arg(long)]
    pub max_passes: Option<u64>,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
        .unwrap_or_default();
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    let home_dir = crate::harness::home_dir_from_ctx(ctx);

    let (source_root, version, source_json) =
        resolve_sync_source_and_version(ctx, &state, &home_dir, &opencode_dir)?;

    if args.watch {
        let opencode_manifest = InstallManifest::load(&opencode_dir).ok();
        return run_watch(
            ctx,
            args,
            &source_root,
            &version,
            &source_json,
            opencode_manifest.as_ref(),
        );
    }
    sync_with(ctx, &source_root, &version, source_json)
}

/// Resolves the source tree, version, and source JSON from an available install manifest,
/// the installed harnesses state, or release provenance.
fn resolve_sync_source_and_version(
    _ctx: &Context,
    state: &State,
    home_dir: &Path,
    opencode_dir: &Path,
) -> Result<(PathBuf, String, serde_json::Value), CeError> {
    // 1. If OpenCode install manifest exists, use it.
    if let Ok(manifest) = InstallManifest::load(opencode_dir) {
        let source_root = resolve_source_root(&manifest.source)?;
        return Ok((source_root, manifest.version, manifest.source));
    }

    // 2. Try loading install-manifest.json from other installed harnesses
    for entry in &state.installed_harnesses {
        if let Some(h_name) = entry["name"].as_str() {
            if let Ok(h_kind) = h_name.parse::<HarnessKind>() {
                let config_dir = if h_kind == HarnessKind::Custom {
                    entry
                        .get("custom")
                        .and_then(|c| c.get("plugins_dir"))
                        .and_then(|p| p.as_str())
                        .map(PathBuf::from)
                } else if let Some(target_dir) = entry.get("target_dir").and_then(|t| t.as_str()) {
                    Some(PathBuf::from(target_dir))
                } else {
                    Some(h_kind.harness_dir(home_dir))
                };
                if let Some(cfg_dir) = config_dir {
                    if let Ok(m) = InstallManifest::load(&cfg_dir) {
                        let source_root = resolve_source_root(&m.source)?;
                        return Ok((source_root, m.version, m.source));
                    }
                }
            }
        }
    }

    // 3. Check source and version recorded directly in state.installed_harnesses
    for entry in &state.installed_harnesses {
        if let (Some(source), Some(version)) = (
            entry.get("source"),
            entry.get("version").and_then(|v| v.as_str()),
        ) {
            if !source.is_null() {
                if let Ok(root) = resolve_source_root(source) {
                    return Ok((root, version.to_string(), source.clone()));
                }
            }
        }
    }

    // 4. Check release provenance
    if let Some(prov) = &state.release_provenance {
        let source_json = serde_json::json!({
            "kind": "github-release",
            "tag": prov.tag,
            "tree": prov.extraction_path,
        });
        if let Ok(root) = resolve_source_root(&source_json) {
            return Ok((root, prov.tag.clone(), source_json));
        }
    }

    // 5. Fail-fast with clear error if no harnesses are installed
    let mut total_harnesses = state
        .installed_harnesses
        .iter()
        .filter_map(|h| h["name"].as_str())
        .count();
    if total_harnesses == 0 {
        total_harnesses = HarnessKind::detect_ce_installed_harnesses(home_dir).len();
    }
    if total_harnesses == 0 {
        return Err(CeError::Runtime(
            "no harnesses installed — run ce-ai install first".into(),
        ));
    }

    Err(CeError::Runtime(
        "no install-manifest.json — run install first".into(),
    ))
}

/// Resolves the source tree recorded in the manifest (local path or the
/// extracted release tree recorded by upgrade).
fn resolve_source_root(source: &serde_json::Value) -> Result<PathBuf, CeError> {
    let root = match source["kind"].as_str() {
        Some("local") => source["path"].as_str(),
        Some("github-release") => source["tree"].as_str(),
        _ => None,
    }
    .ok_or_else(|| CeError::Runtime("cannot resolve source tree from install manifest".into()))?;
    let path = PathBuf::from(root);
    if !path.is_dir() {
        return Err(CeError::Runtime(format!(
            "source tree not found at {} — re-run install or upgrade",
            path.display()
        )));
    }
    Ok(path)
}

/// Shared sync core used by `sync` and `upgrade`: diff desired vs manifest vs
/// filesystem, then plan (dry-run) or apply and rewrite the manifest + state.
pub(crate) fn sync_with(
    ctx: &Context,
    source_root: &Path,
    version: &str,
    source_json: serde_json::Value,
) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state =
        State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())?;
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    let home_dir = crate::harness::home_dir_from_ctx(ctx);

    let mut active_harnesses: Vec<String> = state
        .installed_harnesses
        .iter()
        .filter_map(|h| h["name"].as_str().map(|s| s.to_string()))
        .collect();

    for h in HarnessKind::detect_ce_installed_harnesses(&home_dir) {
        let name = h.to_string();
        if !active_harnesses.contains(&name) {
            active_harnesses.push(name);
        }
    }

    let opencode_manifest = InstallManifest::load(&opencode_dir).ok();
    let opencode_active =
        active_harnesses.iter().any(|h| h == "opencode") || opencode_manifest.is_some();
    if opencode_active && !active_harnesses.contains(&"opencode".to_string()) {
        active_harnesses.push("opencode".to_string());
    }

    if active_harnesses.is_empty() {
        return Err(CeError::Runtime(
            "no harnesses installed — run ce-ai install first".into(),
        ));
    }

    let managed_dir = opencode_dir.join(MANAGED_DIR);

    // Desired: managed-rel path -> sha256, plus the source-rel path to copy from.
    let mut desired: BTreeMap<String, String> = BTreeMap::new();
    let mut source_rel: BTreeMap<String, String> = BTreeMap::new();
    for (managed_rel, (src_rel, hash)) in managed_tree(source_root)? {
        desired.insert(managed_rel.clone(), hash);
        source_rel.insert(managed_rel, src_rel);
    }

    // Retirement respect (R13): once an opencode surface is adopted, the
    // managed-dir skills tree stays retired — sync must not re-harvest it.
    let skip_managed_skills_harvest = state
        .skill_surfaces
        .iter()
        .any(|s| s.harness == "opencode" && s.status == "adopted");
    if skip_managed_skills_harvest {
        desired.retain(|k, _| !k.starts_with("skills/"));
        source_rel.retain(|k, _| !k.starts_with("skills/"));
    }

    let opencode_plan = if opencode_active {
        let installed: BTreeMap<String, String> = opencode_manifest
            .as_ref()
            .map(|m| {
                m.files
                    .iter()
                    .map(|f| (f.path.clone(), f.sha256.clone()))
                    .collect()
            })
            .unwrap_or_default();
        Some(diff::diff(&desired, &installed, &managed_dir))
    } else {
        None
    };

    if ctx.dry_run {
        if let Some(plan) = &opencode_plan {
            if plan.actions.is_empty() {
                println!("plan: no changes");
            }
            for action in &plan.actions {
                let (verb, path) = plan_verb(action);
                println!("plan: {verb} {path}");
            }
        } else if !ctx.quiet {
            println!("plan: no changes");
        }
        return Ok(());
    }

    // Transactional journal (#166): tracked mutations record prior content;
    // state.json remains the final persisted write.
    let mut journal = if ctx.dry_run {
        None
    } else {
        Some(crate::state::journal::Journal::begin(
            &ctx.config_dir,
            "sync",
        )?)
    };
    macro_rules! arm {
        ($p:expr) => {
            if let Some(j) = journal.as_mut() {
                j.arm($p)?;
            }
        };
    }

    if opencode_active {
        if let Some(plan) = &opencode_plan {
            for action in &plan.actions {
                match action {
                    Action::Copy { path } | Action::Restore { path } => {
                        let verb = if matches!(action, Action::Copy { .. }) {
                            "copy"
                        } else {
                            "restore"
                        };
                        arm!(&managed_dir.join(path));
                        let src = source_root.join(&source_rel[path]);
                        write_atomic(&managed_dir.join(path), &std::fs::read(&src)?)?;
                        println!("sync: {verb} {path}");
                    }
                    Action::Remove { path } => {
                        arm!(&managed_dir.join(path));
                        std::fs::remove_file(managed_dir.join(path))?;
                        println!("sync: remove {path}");
                    }
                }
            }
            if plan.actions.is_empty() && !ctx.quiet {
                println!("sync: up to date");
            }
        }

        // Rewrite the manifest with refreshed hashes and version/source (SU-2).
        let files: Vec<ManifestFile> = desired
            .iter()
            .map(|(path, sha256)| ManifestFile {
                path: path.clone(),
                sha256: sha256.clone(),
            })
            .collect();
        arm!(&opencode_dir.join(MANAGED_DIR).join("install-manifest.json"));
        InstallManifest {
            version: version.to_string(),
            plugin_name: opencode_manifest
                .as_ref()
                .map(|m| m.plugin_name.clone())
                .unwrap_or_else(|| "compound-engineering".into()),
            installed_at: opencode_manifest
                .as_ref()
                .map(|m| m.installed_at.clone())
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
            source: source_json.clone(),
            files,
            config_mutations: opencode_manifest
                .as_ref()
                .map(|m| m.config_mutations.clone())
                .unwrap_or_default(),
        }
        .write(&opencode_dir)?;
    }

    // Refresh and sync across all active host-detected and registered harness entries in state.json (SU-2, SU-5).
    let prior_custom: BTreeMap<String, serde_json::Value> = state
        .installed_harnesses
        .iter()
        .filter(|h| h.get("custom").is_some_and(|c| c.is_object()))
        .filter_map(|h| {
            h["name"]
                .as_str()
                .map(|n| (n.to_string(), h["custom"].clone()))
        })
        .collect();

    let prior_entries: BTreeMap<String, serde_json::Value> = state
        .installed_harnesses
        .iter()
        .filter_map(|h| h["name"].as_str().map(|n| (n.to_string(), h.clone())))
        .collect();

    state.installed_harnesses.clear();
    for name in &active_harnesses {
        if let Ok(h_kind) = name.parse::<HarnessKind>() {
            let config_dir = if h_kind == HarnessKind::Opencode {
                opencode_dir.clone()
            } else {
                h_kind.harness_dir(&home_dir)
            };
            let target_config = h_kind.config_path(&config_dir);
            if h_kind == HarnessKind::Custom {
                let Some(cfg) = prior_custom
                    .get(name)
                    .and_then(CustomHarnessConfig::from_state_json)
                else {
                    return Err(CeError::Runtime(
                        "custom harness entry lacks its directory snapshot; \
                         re-run 'ce-ai install --harness custom'"
                            .into(),
                    ));
                };
                std::fs::create_dir_all(&cfg.plugins_dir)?;
                std::fs::create_dir_all(&cfg.skills_dir)?;
                let mut files: Vec<ManifestFile> = Vec::new();
                for (rel, src_rel) in &source_rel {
                    let dest = if let Some(rest) = custom_plugin_rel(rel) {
                        cfg.plugins_dir.join(rest)
                    } else if let Some(rest) = custom_skill_rel(rel) {
                        cfg.skills_dir.join(rest)
                    } else {
                        continue;
                    };
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    arm!(&dest);
                    write_atomic(&dest, &std::fs::read(source_root.join(src_rel))?)?;
                    files.push(ManifestFile {
                        path: rel.clone(),
                        sha256: desired[rel].clone(),
                    });
                }
                let prior_mutations = InstallManifest::load(&cfg.plugins_dir)
                    .map(|m| m.config_mutations)
                    .unwrap_or_default();
                InstallManifest {
                    version: version.to_string(),
                    plugin_name: "compound-engineering".into(),
                    installed_at: Utc::now().to_rfc3339(),
                    source: source_json.clone(),
                    files,
                    config_mutations: prior_mutations,
                }
                .write(&cfg.plugins_dir)?;
            } else if let Some(spec) = registration_spec(h_kind) {
                // Strategy table: one exhaustive entry per table-driven kind
                // (see harness::registration). Skills are never copied into
                // harness-owned directories — adoption is the only delivery
                // path (token-neutrality, R4).
                spec.register_companions(&target_config)?;
                let manifest_path = config_dir.join(MANAGED_DIR).join("install-manifest.json");
                if manifest_path.exists() {
                    arm!(&manifest_path);
                    let _ = InstallManifest {
                        version: version.to_string(),
                        plugin_name: "compound-engineering".into(),
                        installed_at: Utc::now().to_rfc3339(),
                        source: source_json.clone(),
                        files: vec![],
                        config_mutations: vec![],
                    }
                    .write(&config_dir);
                }
            } else if h_kind == HarnessKind::Opencode {
                // OpenCode's own registration: plugin entry + skills paths.
                crate::opencode::config::ensure_plugin_and_skills(
                    &target_config,
                    &crate::opencode::plugins::plugin_entry(&config_dir).to_string_lossy(),
                    &crate::opencode::plugins::skills_path(&config_dir).to_string_lossy(),
                )?;
            } else {
                // Every supported kind has an explicit arm above; reaching
                // this point means state.json references an unsupported
                // harness and OpenCode-format mutations must never be a
                // fallback (invariant #5).
                return Err(CeError::Runtime(format!(
                    "cannot re-sync unsupported harness '{name}' recorded in state.json"
                )));
            }
        }
        let mut entry = serde_json::json!({
            "name": name,
            "version": version.to_string(),
            "source": source_json.clone(),
            "last_synced_at": Utc::now().to_rfc3339(),
        });
        if let Some(prior) = prior_entries.get(name) {
            if let Some(scope) = prior.get("scope") {
                entry["scope"] = scope.clone();
            }
            if let Some(target_dir) = prior.get("target_dir") {
                entry["target_dir"] = target_dir.clone();
            }
            if let Some(installed_at) = prior.get("installed_at") {
                entry["installed_at"] = installed_at.clone();
            }
        } else if name == "opencode" && opencode_dir != ctx.opencode_config_dir {
            entry["scope"] = serde_json::json!("workspace");
            entry["target_dir"] = serde_json::json!(opencode_dir.display().to_string());
        }
        if let Some(custom) = prior_custom.get(name) {
            entry["custom"] = custom.clone();
        }
        state.installed_harnesses.push(entry);
    }
    // Repair model-assignment desync: import effective opencode.json
    // assignments into state.json (config→state; #111). Config is the live
    // truth — state is never pushed back over user-edited config here.
    if opencode_active {
        let opencode_json = opencode_dir.join("opencode.json");
        if opencode_json.exists() {
            let config = crate::opencode::config::read_config(&opencode_json)?;
            for (slot, model) in
                crate::commands::models::import_config_assignments(&mut state, &config)
            {
                if !ctx.quiet {
                    println!("sync: imported model {slot} = {model}");
                }
            }
            for slot in crate::commands::models::purge_stale_assignments(&mut state, &config) {
                if !ctx.quiet {
                    println!("sync: purged stale assignment {slot}");
                }
            }
        }
    }

    // Adopted surfaces: rewrite drift back to canonical (U4/R16) and flag
    // vanished roots as orphaned (R19). Runs before the state save so the
    // ledger status changes persist.
    let mut restored_drift: Vec<String> = Vec::new();
    let mut orphaned: Vec<String> = Vec::new();
    let adopted: Vec<crate::state::state::SkillSurface> = state
        .skill_surfaces
        .iter()
        .filter(|s| s.status == "adopted")
        .cloned()
        .collect();
    for surface in &adopted {
        if !surface.root.exists() {
            if let Some(entry) = state
                .skill_surfaces
                .iter_mut()
                .find(|s| s.root == surface.root && s.harness == surface.harness)
            {
                entry.status = "orphaned".to_string();
            }
            orphaned.push(format!("{} ({})", surface.harness, surface.root.display()));
            continue;
        }
        for f in &surface.files {
            let dest = surface.root.join(&f.path);
            let current = std::fs::read(&dest)
                .map(|bytes| crate::state::diff::sha256_hex(&bytes))
                .ok();
            if current.as_deref() == Some(f.sha256.as_str()) {
                continue;
            }
            let _ = crate::state::backups::backup_file(&ctx.config_dir.join("backups"), &dest);
            arm!(&dest);
            let content = std::fs::read(source_root.join("skills").join(&f.path))?;
            crate::state::write_atomic(&dest, &content)?;
            restored_drift.push(format!("{}/{}", surface.harness, f.path));
        }
    }

    state.save(&state_path)?;
    if let Some(j) = journal.take() {
        j.complete()?;
    }

    if !ctx.dry_run {
        for project in &state.projects {
            if project.path.exists() {
                let inner_body = crate::commands::init_prj::render_block_content(project.tier);
                let _ = crate::commands::init_prj::reconcile_project_harness_hooks(
                    &project.path,
                    inner_body,
                );
            }
        }

        if let Err(e) = crate::source::registry::SkillRegistry::sync_registry(ctx) {
            if !ctx.quiet {
                eprintln!("warning: skill registry sync failed: {e}");
            }
        }
    }

    // Issue #161: report only what was actually verified. The OpenCode
    // managed surface is re-hashed after apply; harness skill copies are
    // hash-checked when performed; registration-only adapters are labelled
    // as not verified. Any verified-surface drift fails with exit code 6.
    if !ctx.dry_run {
        let mut surfaces: Vec<SurfaceCheck> = Vec::new();

        if opencode_active {
            let drift = verify_tree_against(&managed_dir, &desired);
            surfaces.push(SurfaceCheck {
                harness: "opencode".into(),
                status: CheckStatus::from_drift(desired.len(), drift),
            });
        }

        let skills_expected: BTreeMap<String, String> = desired
            .iter()
            .filter(|(path, _)| path.starts_with("skills/"))
            .map(|(path, hash)| (path.trim_start_matches("skills/").to_string(), hash.clone()))
            .collect();
        let plugins_expected: BTreeMap<String, String> = desired
            .iter()
            .filter(|(path, _)| path.starts_with("plugins/"))
            .map(|(path, hash)| {
                (
                    path.trim_start_matches("plugins/").to_string(),
                    hash.clone(),
                )
            })
            .collect();
        // Adoption-aware surfaces (U5): a ledger-tracked adopted surface is
        // the harness's managed surface; untracked adoptable content renders
        // as pending-adoption; marketplace/plugin-cache CE copies render as
        // external-duplicate (R17/R18/R19).
        let ledger_roots: Vec<(String, PathBuf)> = state
            .skill_surfaces
            .iter()
            .filter(|s| s.status == "adopted")
            .map(|s| (s.harness.clone(), s.root.clone()))
            .collect();
        let pending = crate::commands::adopt::pending_adoptions(ctx, &home_dir, &ledger_roots);
        let external = crate::commands::adopt::external_duplicates(&home_dir);
        for name in &active_harnesses {
            if let Some(surface) = state
                .skill_surfaces
                .iter()
                .find(|s| s.harness == *name && (s.status == "adopted" || s.status == "orphaned"))
            {
                let status = if surface.status == "orphaned" || !surface.root.exists() {
                    CheckStatus::Orphaned
                } else {
                    let expected: BTreeMap<String, String> = surface
                        .files
                        .iter()
                        .map(|f| (f.path.clone(), f.sha256.clone()))
                        .collect();
                    let drift = verify_tree_against(&surface.root, &expected);
                    CheckStatus::from_drift(surface.files.len(), drift)
                };
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status,
                });
                continue;
            }
            if pending.iter().any(|(h, _)| h == name) {
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status: CheckStatus::PendingAdoption,
                });
                continue;
            }
            if name == "opencode" {
                continue;
            }
            let Ok(kind) = name.parse::<HarnessKind>() else {
                continue;
            };
            if kind == HarnessKind::Custom {
                match prior_custom
                    .get(name)
                    .and_then(CustomHarnessConfig::from_state_json)
                {
                    Some(cfg) => {
                        if desired.is_empty() {
                            surfaces.push(SurfaceCheck {
                                harness: name.clone(),
                                status: CheckStatus::NotVerified {
                                    reason: REASON_NO_MANAGED_FILES,
                                },
                            });
                            continue;
                        }
                        // Both custom trees are hash-checked like native
                        // skills surfaces; drift on either fails sync.
                        let mut drift = verify_tree_against(&cfg.plugins_dir, &plugins_expected);
                        let skill_drift = verify_tree_against(&cfg.skills_dir, &skills_expected);
                        drift.missing.extend(skill_drift.missing);
                        drift.mismatched.extend(skill_drift.mismatched);
                        surfaces.push(SurfaceCheck {
                            harness: name.clone(),
                            status: CheckStatus::from_drift(desired.len(), drift),
                        });
                    }
                    None => surfaces.push(SurfaceCheck {
                        harness: name.clone(),
                        status: CheckStatus::NotVerified {
                            reason: REASON_NO_SNAPSHOT,
                        },
                    }),
                }
            } else if matches!(
                kind,
                HarnessKind::Claude
                    | HarnessKind::Codex
                    | HarnessKind::Copilot
                    | HarnessKind::Grok
                    | HarnessKind::Kimi
                    | HarnessKind::Agy
                    | HarnessKind::Pi
                    | HarnessKind::Fx
            ) {
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status: CheckStatus::NotVerified {
                        reason: REASON_NO_MANAGED_SKILLS,
                    },
                });
            } else {
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status: CheckStatus::NotVerified {
                        reason: REASON_CONFIG_ONLY,
                    },
                });
            }
        }
        if !external.is_empty() {
            surfaces.push(SurfaceCheck {
                harness: "external-duplicate".into(),
                status: CheckStatus::ExternalDuplicate { paths: external },
            });
        }

        if !ctx.quiet {
            let source_kind = source_json
                .get("kind")
                .and_then(|k| k.as_str())
                .unwrap_or("unknown");
            println!("== [Sync Verification Matrix] ==");
            println!("version: {version}");
            println!("source: {source_kind}");
            for surface in &surfaces {
                println!("{}", matrix_line(&surface.harness, &surface.status));
                if let CheckStatus::Failed {
                    mismatched,
                    missing,
                } = &surface.status
                {
                    for line in failed_detail_lines(mismatched, missing) {
                        println!("{line}");
                    }
                }
            }
            let verified = surfaces
                .iter()
                .filter(|s| matches!(s.status, CheckStatus::Verified { .. }))
                .count();
            let failed = surfaces
                .iter()
                .filter(|s| matches!(s.status, CheckStatus::Failed { .. }))
                .count();
            let unverified = surfaces
                .iter()
                .filter(|s| matches!(s.status, CheckStatus::NotVerified { .. }))
                .count();
            println!("{}", reconciliation_line(verified, unverified, failed));
            for path in &restored_drift {
                println!("  ↻ restored-drift: {path}");
            }
            if unverified > 0 {
                for line in guidance_note_lines() {
                    println!("{line}");
                }
            }
        }

        let failed_surfaces: Vec<String> = surfaces
            .iter()
            .filter_map(|s| match &s.status {
                CheckStatus::Failed {
                    mismatched,
                    missing,
                } => Some(format!(
                    "{} ({} drifted)",
                    s.harness,
                    mismatched.len() + missing.len()
                )),
                _ => None,
            })
            .collect();
        if !failed_surfaces.is_empty() {
            return Err(CeError::Verification(format!(
                "sync verification failed for {}",
                failed_surfaces.join(", ")
            )));
        }
    }

    Ok(())
}

/// Drift found by [`verify_tree_against`]: files absent from disk or whose
/// on-disk hash no longer matches the expected digest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TreeDrift {
    pub missing: Vec<String>,
    pub mismatched: Vec<String>,
}

impl TreeDrift {
    pub(crate) fn total(&self) -> usize {
        self.missing.len() + self.mismatched.len()
    }
}

/// Re-hashes every expected file under `root` and reports which ones are
/// missing or hash-mismatched. Pure filesystem reads — never mutates.
pub(crate) fn verify_tree_against(root: &Path, expected: &BTreeMap<String, String>) -> TreeDrift {
    let mut drift = TreeDrift::default();
    for (rel, hash) in expected {
        let path = root.join(rel);
        match std::fs::read(&path) {
            Ok(bytes) => {
                if &sha256_hex(&bytes) != hash {
                    drift.mismatched.push(rel.clone());
                }
            }
            Err(_) => drift.missing.push(rel.clone()),
        }
    }
    drift
}

/// Outcome of one harness surface's post-sync verification (Issue #161):
/// statuses are produced only by checks that actually ran.
#[derive(Debug)]
pub(crate) struct SurfaceCheck {
    pub harness: String,
    pub status: CheckStatus,
}

#[derive(Debug, PartialEq)]
pub(crate) enum CheckStatus {
    /// Every expected file present and hash-matching.
    Verified { matched: usize, total: usize },
    /// Real drift detected on a verified surface.
    Failed {
        mismatched: Vec<String>,
        missing: Vec<String>,
    },
    /// No hash check ran for this surface; the reason says why.
    NotVerified { reason: &'static str },
    /// Adoptable `ce-*` content under a harness skills root the ledger does
    /// not track (R17) — non-failing, adoption is explicit.
    PendingAdoption,
    /// CE copies under marketplace/plugin-cache roots (R18) — reported with
    /// paths, never adopted or modified.
    ExternalDuplicate { paths: Vec<String> },
    /// A ledger-tracked adopted root vanished from disk (R19) — re-adoption
    /// required.
    Orphaned,
}

impl CheckStatus {
    fn from_drift(total: usize, drift: TreeDrift) -> Self {
        if drift.total() == 0 {
            Self::Verified {
                matched: total,
                total,
            }
        } else {
            Self::Failed {
                mismatched: drift.mismatched,
                missing: drift.missing,
            }
        }
    }
}

/// Reason shown for native skills harnesses when the installed source ships
/// no managed skills tree (SVX-1): nothing was hash-verified and that is the
/// expected state, not an error.
const REASON_NO_MANAGED_SKILLS: &str =
    "ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)";

/// Reason shown for registration-only adapters such as cursor (SVX-1).
const REASON_CONFIG_ONLY: &str = "config registration only — no managed assets to hash-verify";

/// Reason shown for custom harnesses whose desired tree is empty.
const REASON_NO_MANAGED_FILES: &str = "no managed files — nothing to hash-verify";

/// Reason shown when a custom harness entry lacks its directory snapshot.
const REASON_NO_SNAPSHOT: &str = "no directory snapshot — re-run 'ce-ai install --harness custom'";

/// Renders one matrix line for a harness surface (SVX-1/SVX-2 wording).
/// `registered` states are healthy: ce-ai manages no files there, so there
/// is nothing to hash-verify.
fn matrix_line(harness: &str, status: &CheckStatus) -> String {
    match status {
        CheckStatus::Verified { matched, total } => {
            format!("  ✓ {harness}: verified — {matched}/{total} managed files match SHA256")
        }
        CheckStatus::Failed {
            mismatched,
            missing,
        } => format!(
            "  ✗ {harness}: FAILED — {} file(s) drifted",
            mismatched.len() + missing.len()
        ),
        CheckStatus::NotVerified { reason } => {
            format!("  ○ {harness}: registered — {reason}")
        }
        CheckStatus::PendingAdoption => {
            format!("  ○ {harness}: pending-adoption — run `skills adopt` to put it under management")
        }
        CheckStatus::ExternalDuplicate { paths } => format!(
            "  ○ external-duplicate — marketplace/plugin-cache CE copies (never adopted; remove manually): {}",
            paths.join(", ")
        ),
        CheckStatus::Orphaned => {
            format!("  ○ {harness}: orphaned — adopted root vanished; re-run `skills adopt`")
        }
    }
}

/// Indented detail lines for a FAILED surface, one per drifted path (SVX-2).
fn failed_detail_lines(mismatched: &[String], missing: &[String]) -> Vec<String> {
    mismatched
        .iter()
        .chain(missing.iter())
        .map(|path| format!("      {path}"))
        .collect()
}

/// Reconciliation summary line (SVX-1): unmanaged surfaces count as
/// `registered (nothing to verify)`, never as `unverified`.
fn reconciliation_line(verified: usize, unverified: usize, failed: usize) -> String {
    format!(
        "reconciliation status: {verified} verified, {unverified} registered (nothing to verify), {failed} failed"
    )
}

/// Newbie guidance printed after the matrix when any surface is unverified
/// (SVX-3): what `registered` means, how to put a harness under ce-ai
/// management, and the verification-scope boundary.
fn guidance_note_lines() -> Vec<String> {
    vec![
        "note: 'registered' = ce-ai wrote harness config only; it manages no files on that"
            .to_string(),
        "      surface, so there is nothing to hash-verify. CE installed via other channels"
            .to_string(),
        "      (plugin marketplaces, manual copies) is outside ce-ai's verification scope."
            .to_string(),
        "      To put a harness under ce-ai management: ce-ai install --harness <name>".to_string(),
        "      (or --harness all). Skill files are managed per harness only when the".to_string(),
        "      installed source ships a managed skills tree.".to_string(),
    ]
}

/// Per-kind managed-skills root used for post-sync hash verification and
/// adoption detection. Agy nests its skills under `config/skills`; every
/// other directory-copying harness uses `<harness_dir>/skills`.
pub(crate) fn sync_skills_root(kind: HarnessKind, home_dir: &Path) -> PathBuf {
    let dir = kind.harness_dir(home_dir);
    if kind == HarnessKind::Agy {
        dir.join("config").join("skills")
    } else {
        dir.join("skills")
    }
}

static INIT_CTRLC: std::sync::Once = std::sync::Once::new();
static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn setup_ctrlc() {
    INIT_CTRLC.call_once(|| {
        if let Err(e) = ctrlc::set_handler(move || {
            RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
        }) {
            eprintln!("warning: could not install Ctrl-C handler: {e}");
        }
    });
    RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
}

pub fn run_watch(
    ctx: &Context,
    args: &Args,
    source_root: &Path,
    version: &str,
    source_json: &serde_json::Value,
    opencode_manifest: Option<&InstallManifest>,
) -> Result<(), CeError> {
    setup_ctrlc();

    let interval = std::time::Duration::from_millis(args.interval_ms.unwrap_or(2000));
    let mut passes = 0;
    let mut repaired_count = 0;

    if !ctx.quiet {
        println!("ce-ai sync --watch: monitoring managed paths for drift...");
    }

    while RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(max) = args.max_passes {
            if passes >= max {
                break;
            }
        }

        if passes > 0 {
            std::thread::sleep(interval);
            if !RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
        }

        match check_and_repair_drift(ctx, source_root, version, source_json, opencode_manifest) {
            Ok(true) => {
                repaired_count += 1;
                if !ctx.quiet {
                    println!(
                        "ce-ai sync --watch: repaired drift at {}",
                        chrono::Utc::now().to_rfc3339()
                    );
                }
            }
            Ok(false) => {}
            Err(err) => {
                eprintln!("notice: sync pass error: {err} — retrying on next pass");
            }
        }
        passes += 1;
    }

    if !ctx.quiet {
        println!(
            "ce-ai sync --watch: stopped after {passes} pass(es) ({repaired_count} drift repair(s))."
        );
    }
    Ok(())
}

fn check_and_repair_drift(
    ctx: &Context,
    source_root: &Path,
    version: &str,
    source_json: &serde_json::Value,
    opencode_manifest: Option<&InstallManifest>,
) -> Result<bool, CeError> {
    let state = State::load(&ctx.config_dir.join("state.json")).unwrap_or_default();
    let opencode_dir = ctx.resolve_opencode_dir(&state);
    let managed_dir = opencode_dir.join(MANAGED_DIR);
    let mut desired: BTreeMap<String, String> = managed_tree(source_root)?
        .into_iter()
        .map(|(managed_rel, (_, hash))| (managed_rel, hash))
        .collect();
    // Retirement respect (R13): the opencode surface is adopted; the
    // managed-dir skills tree stays retired.
    let opencode_adopted = State::load(&ctx.config_dir.join("state.json"))
        .map(|s| {
            s.skill_surfaces
                .iter()
                .any(|s| s.harness == "opencode" && s.status == "adopted")
        })
        .unwrap_or(false);
    if opencode_adopted {
        desired.retain(|k, _| !k.starts_with("skills/"));
    }

    if let Some(manifest) = opencode_manifest {
        let installed: BTreeMap<String, String> = manifest
            .files
            .iter()
            .map(|f| (f.path.clone(), f.sha256.clone()))
            .collect();

        let plan = diff::diff(&desired, &installed, &managed_dir);
        if plan.actions.is_empty() {
            return Ok(false);
        }

        if ctx.dry_run {
            println!(
                "plan: dry-run watch detected {} drift action(s)",
                plan.actions.len()
            );
            for action in &plan.actions {
                let (verb, path) = plan_verb(action);
                println!("plan: {verb} {path}");
            }
            return Ok(false);
        }

        sync_with(ctx, source_root, version, source_json.clone())?;
        return Ok(true);
    }

    Ok(false)
}

fn plan_verb(action: &Action) -> (&'static str, &str) {
    match action {
        Action::Copy { path } => ("copy", path),
        Action::Restore { path } => ("restore", path),
        Action::Remove { path } => ("remove", path),
    }
}

#[cfg(test)]
#[path = "tests/sync.rs"]
mod tests;
