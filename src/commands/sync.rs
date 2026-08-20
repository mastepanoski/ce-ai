//! `ce-ai sync`: reconcile the desired source tree against installed files
//! (SU-1..SU-4). Desired manifest comes from the current source; the diff
//! engine plans copy/restore/remove actions that `--dry-run` only prints.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::manifest::{InstallManifest, ManifestFile};
use crate::opencode::plugins::MANAGED_DIR;
use crate::source::cache::read_local_tree;
use crate::state::diff::{self, Action};
use crate::state::state::State;
use crate::state::write_atomic;

/// Source-tree dirs ce-ai manages; the `.opencode/` prefix is stripped on copy
/// (mirrors the install command's filter).
const MANAGED_PREFIXES: [&str; 2] = [".opencode/plugins", ".opencode/skills"];

pub fn run(ctx: &Context) -> Result<(), CeError> {
    let manifest = InstallManifest::load(&ctx.opencode_config_dir)
        .map_err(|_| CeError::Runtime("no install-manifest.json — run install first".into()))?;
    let source_root = resolve_source_root(&manifest.source)?;
    sync_with(ctx, &source_root, &manifest.version, manifest.source.clone())
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
    let installed: BTreeMap<String, String> =
        manifest.files.iter().map(|f| (f.path.clone(), f.sha256.clone())).collect();

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
                let verb = if matches!(action, Action::Copy { .. }) { "copy" } else { "restore" };
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
        .map(|(path, sha256)| ManifestFile { path: path.clone(), sha256: sha256.clone() })
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

    // Refresh the opencode harness entry in state.json (SU-2, SU-5).
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    if let Some(harness) = state.installed_harnesses.iter_mut().find(|h| h["name"].as_str() == Some("opencode")) {
        harness["version"] = serde_json::Value::String(version.to_string());
        harness["source"] = source_json;
        harness["last_synced_at"] = serde_json::Value::String(Utc::now().to_rfc3339());
    }
    state.save(&state_path)?;
    Ok(())
}

fn plan_verb(action: &Action) -> (&'static str, &str) {
    match action {
        Action::Copy { path } => ("copy", path),
        Action::Restore { path } => ("restore", path),
        Action::Remove { path } => ("remove", path),
    }
}