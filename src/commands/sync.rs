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
use crate::harness::registration::{copy_managed_skills, registration_spec};
use crate::harness::HarnessKind;
use crate::opencode::manifest::{InstallManifest, ManifestFile};
use crate::opencode::plugins::MANAGED_DIR;
use crate::source::cache::read_local_tree;
use crate::state::diff::{self, sha256_hex, Action};
use crate::state::state::State;
use crate::state::write_atomic;

/// Source-tree dirs ce-ai manages; the `.opencode/` prefix is stripped on copy
/// (mirrors the install command's filter).
const MANAGED_PREFIXES: [&str; 2] = [".opencode/plugins", ".opencode/skills"];

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
    let manifest = InstallManifest::load(&ctx.opencode_config_dir)
        .map_err(|_| CeError::Runtime("no install-manifest.json — run install first".into()))?;
    let source_root = resolve_source_root(&manifest.source)?;
    if args.watch {
        return run_watch(ctx, args, &source_root, &manifest);
    }
    sync_with(
        ctx,
        &source_root,
        &manifest.version,
        manifest.source.clone(),
    )
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
    let manifest = InstallManifest::load(&ctx.opencode_config_dir)?;
    let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);

    // Desired: managed-rel path -> sha256, plus the source-rel path to copy from.
    let mut desired: BTreeMap<String, String> = BTreeMap::new();
    let mut source_rel: BTreeMap<String, String> = BTreeMap::new();
    for (rel, hash) in read_local_tree(source_root)? {
        if MANAGED_PREFIXES.iter().any(|p| rel.starts_with(p)) {
            let managed_rel = rel.trim_start_matches(".opencode/").to_string();
            desired.insert(managed_rel.clone(), hash);
            source_rel.insert(managed_rel, rel);
        }
    }
    let installed: BTreeMap<String, String> = manifest
        .files
        .iter()
        .map(|f| (f.path.clone(), f.sha256.clone()))
        .collect();

    let plan = diff::diff(&desired, &installed, &managed_dir);

    if ctx.dry_run {
        if plan.actions.is_empty() {
            println!("plan: no changes");
        }
        for action in &plan.actions {
            let (verb, path) = plan_verb(action);
            println!("plan: {verb} {path}");
        }
        return Ok(());
    }

    for action in &plan.actions {
        match action {
            Action::Copy { path } | Action::Restore { path } => {
                let verb = if matches!(action, Action::Copy { .. }) {
                    "copy"
                } else {
                    "restore"
                };
                let src = source_root.join(&source_rel[path]);
                write_atomic(&managed_dir.join(path), &std::fs::read(&src)?)?;
                println!("sync: {verb} {path}");
            }
            Action::Remove { path } => {
                std::fs::remove_file(managed_dir.join(path))?;
                println!("sync: remove {path}");
            }
        }
    }
    if plan.actions.is_empty() && !ctx.quiet {
        println!("sync: up to date");
    }

    // Rewrite the manifest with refreshed hashes and version/source (SU-2).
    let files: Vec<ManifestFile> = desired
        .iter()
        .map(|(path, sha256)| ManifestFile {
            path: path.clone(),
            sha256: sha256.clone(),
        })
        .collect();
    InstallManifest {
        version: version.to_string(),
        plugin_name: manifest.plugin_name.clone(),
        installed_at: manifest.installed_at.clone(),
        source: source_json.clone(),
        files,
        config_mutations: manifest.config_mutations.clone(),
    }
    .write(&ctx.opencode_config_dir)?;

    // Refresh and sync across all active host-detected and registered harness entries in state.json (SU-2, SU-5).
    let state_path = ctx.config_dir.join("state.json");
    let mut state =
        State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())?;

    let mut active_harnesses: Vec<String> = state
        .installed_harnesses
        .iter()
        .filter_map(|h| h["name"].as_str().map(|s| s.to_string()))
        .collect();

    if let Ok(home) = std::env::var("HOME") {
        let home_path = Path::new(&home);
        for h in HarnessKind::detect_ce_installed_harnesses(home_path) {
            let name = h.to_string();
            if !active_harnesses.contains(&name) {
                active_harnesses.push(name);
            }
        }
    }
    if active_harnesses.is_empty() {
        active_harnesses.push("opencode".to_string());
    }

    let home_dir = crate::harness::home_dir_from_ctx(ctx);

    // Custom-mode directory snapshots must survive the state rebuild below.
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

    state.installed_harnesses.clear();
    for name in &active_harnesses {
        if let Ok(h_kind) = name.parse::<HarnessKind>() {
            let config_dir = if h_kind == HarnessKind::Opencode {
                ctx.opencode_config_dir.clone()
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
                    write_atomic(&dest, &std::fs::read(source_root.join(src_rel))?)?;
                    files.push(ManifestFile {
                        path: rel.clone(),
                        sha256: desired[rel].clone(),
                    });
                }
                InstallManifest {
                    version: version.to_string(),
                    plugin_name: "compound-engineering".into(),
                    installed_at: Utc::now().to_rfc3339(),
                    source: source_json.clone(),
                    files,
                    config_mutations: manifest.config_mutations.clone(),
                }
                .write(&cfg.plugins_dir)?;
            } else if let Some(spec) = registration_spec(h_kind) {
                // Strategy table: one exhaustive entry per table-driven kind
                // (see harness::registration).
                spec.register_companions(&target_config)?;
                if let Some(subpath) = spec.skills_subpath {
                    copy_managed_skills(&managed_dir, &config_dir.join(subpath))?;
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
        if let Some(custom) = prior_custom.get(name) {
            entry["custom"] = custom.clone();
        }
        state.installed_harnesses.push(entry);
    }
    // Repair model-assignment desync: import effective opencode.json
    // assignments into state.json (config→state; #111). Config is the live
    // truth — state is never pushed back over user-edited config here.
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    let config = crate::opencode::config::read_config(&opencode_json)?;
    for (slot, model) in crate::commands::models::import_config_assignments(&mut state, &config) {
        if !ctx.quiet {
            println!("sync: imported model {slot} = {model}");
        }
    }
    for slot in crate::commands::models::purge_stale_assignments(&mut state, &config) {
        if !ctx.quiet {
            println!("sync: purged stale assignment {slot}");
        }
    }
    state.save(&state_path)?;

    if !ctx.dry_run {
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

        let drift = verify_tree_against(&managed_dir, &desired);
        surfaces.push(SurfaceCheck {
            harness: "opencode".into(),
            status: CheckStatus::from_drift(desired.len(), drift),
        });

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
        for name in &active_harnesses {
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
                                    reason: "no managed tree present",
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
                            reason: "no directory snapshot",
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
                let skills_dir = sync_skills_root(kind, &home_dir);
                if skills_expected.is_empty() {
                    surfaces.push(SurfaceCheck {
                        harness: name.clone(),
                        status: CheckStatus::NotVerified {
                            reason: "no managed skills tree present",
                        },
                    });
                    continue;
                }
                let drift = verify_tree_against(&skills_dir, &skills_expected);
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status: CheckStatus::from_drift(skills_expected.len(), drift),
                });
            } else {
                surfaces.push(SurfaceCheck {
                    harness: name.clone(),
                    status: CheckStatus::NotVerified {
                        reason: "config registration only — asset hashes not checked",
                    },
                });
            }
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
                match &surface.status {
                    CheckStatus::Verified { matched, total } => println!(
                        "  ✓ {harness}: verified — {matched}/{total} files match SHA256",
                        harness = surface.harness,
                    ),
                    CheckStatus::Failed {
                        mismatched,
                        missing,
                    } => {
                        println!(
                            "  ✗ {harness}: FAILED — {count} file(s) drifted",
                            harness = surface.harness,
                            count = mismatched.len() + missing.len()
                        );
                        for path in mismatched.iter().chain(missing.iter()) {
                            println!("      {path}");
                        }
                    }
                    CheckStatus::NotVerified { reason } => println!(
                        "  ○ {harness}: synced — verification not performed ({reason})",
                        harness = surface.harness,
                    ),
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
            println!(
                "reconciliation status: {verified} verified, {unverified} unverified, {failed} failed"
            );
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
#[derive(Debug, Default, PartialEq)]
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

/// Per-kind managed-skills root used for post-sync hash verification.
/// Agy nests its skills under `config/skills`; every other directory-copying
/// harness uses `<harness_dir>/skills`.
fn sync_skills_root(kind: HarnessKind, home_dir: &Path) -> PathBuf {
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
    manifest: &InstallManifest,
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

        match check_and_repair_drift(ctx, source_root, manifest) {
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
    manifest: &InstallManifest,
) -> Result<bool, CeError> {
    let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);
    let mut desired: BTreeMap<String, String> = BTreeMap::new();
    for (rel, hash) in read_local_tree(source_root)? {
        if MANAGED_PREFIXES.iter().any(|p| rel.starts_with(p)) {
            let managed_rel = rel.trim_start_matches(".opencode/").to_string();
            desired.insert(managed_rel, hash);
        }
    }
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

    sync_with(ctx, source_root, &manifest.version, manifest.source.clone())?;
    Ok(true)
}

fn plan_verb(action: &Action) -> (&'static str, &str) {
    match action {
        Action::Copy { path } => ("copy", path),
        Action::Restore { path } => ("restore", path),
        Action::Remove { path } => ("remove", path),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tempfile::tempdir;

    use super::{sync_skills_root, verify_tree_against, CheckStatus, TreeDrift};
    use crate::harness::registration::registration_spec;
    use crate::harness::HarnessKind;
    use crate::state::diff::sha256_hex;

    #[test]
    fn registration_specs_cover_the_table_driven_kinds() {
        use HarnessKind::*;
        for kind in [Claude, Codex, Copilot, Grok, Kimi, Agy, Fx] {
            let spec = registration_spec(kind).expect("table-driven kind");
            assert!(spec.register_mcp.is_some());
            assert!(spec.skills_subpath.is_some());
            if kind == Agy {
                assert_eq!(spec.skills_subpath, Some("config/skills"));
            }
        }

        // Pi: skills tree only — No-MCP by design (objective 8).
        let pi = registration_spec(Pi).expect("pi spec");
        assert!(pi.register_mcp.is_none());
        assert_eq!(pi.skills_subpath, Some("skills"));

        // Cursor consumes MCP servers only; copying a skills tree into its
        // directory would pollute user storage (regression pin).
        let cursor = registration_spec(Cursor).expect("cursor spec");
        assert!(cursor.register_mcp.is_some());
        assert_eq!(cursor.skills_subpath, None);

        for kind in [Opencode, Custom, Deepseek] {
            assert!(registration_spec(kind).is_none(), "dedicated arm kind");
        }
    }

    #[test]
    fn sync_skills_root_nests_agy_under_config() {
        let home = tempdir().unwrap();
        let dir = home.path();
        assert_eq!(
            sync_skills_root(HarnessKind::Agy, dir),
            dir.join(".gemini").join("config").join("skills")
        );
        assert_eq!(
            sync_skills_root(HarnessKind::Pi, dir),
            dir.join(".pi").join("agent").join("skills")
        );
    }

    fn expected_map(files: &[(&str, &[u8])]) -> BTreeMap<String, String> {
        files
            .iter()
            .map(|(name, bytes)| ((*name).to_string(), sha256_hex(bytes)))
            .collect()
    }

    #[test]
    fn clean_tree_reports_no_drift() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"alpha").unwrap();
        let expected = expected_map(&[("a.txt", b"alpha")]);

        let drift = verify_tree_against(dir.path(), &expected);
        assert_eq!(drift, TreeDrift::default());
        assert_eq!(
            CheckStatus::from_drift(1, drift),
            CheckStatus::Verified {
                matched: 1,
                total: 1
            }
        );
    }

    #[test]
    fn hash_mismatch_is_detected_per_file() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), b"tampered").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"beta").unwrap();
        let expected = expected_map(&[("a.txt", b"alpha"), ("b.txt", b"beta")]);

        let drift = verify_tree_against(dir.path(), &expected);
        assert_eq!(drift.mismatched, vec!["a.txt".to_string()]);
        assert!(drift.missing.is_empty());
    }

    #[test]
    fn missing_files_are_reported_separately() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("b.txt"), b"beta").unwrap();
        let expected = expected_map(&[("a.txt", b"alpha"), ("b.txt", b"beta")]);

        let drift = verify_tree_against(dir.path(), &expected);
        assert!(drift.mismatched.is_empty());
        assert_eq!(drift.missing, vec!["a.txt".to_string()]);

        let status = CheckStatus::from_drift(2, drift);
        match status {
            CheckStatus::Failed {
                mismatched,
                missing,
            } => {
                assert!(mismatched.is_empty());
                assert_eq!(missing.len(), 1);
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn nested_paths_are_hashed_relative_to_root() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("skills/ce-work")).unwrap();
        std::fs::write(dir.path().join("skills/ce-work/SKILL.md"), b"# skill").unwrap();
        let expected = expected_map(&[("ce-work/SKILL.md", b"# skill")]);

        // The harness skills root maps `skills/<rest>` onto `<root>/<rest>`.
        let skills_root = dir.path().join("skills");
        let drift = verify_tree_against(&skills_root, &expected);
        assert_eq!(drift, TreeDrift::default());
    }
}
