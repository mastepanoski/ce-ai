//! `ce-ai install`: resolve source, plan, back up, then apply (OI-1..OI-5, SU-4).

use std::collections::BTreeMap;
use std::path::PathBuf;

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::ensure_plugin_and_skills;
use crate::opencode::manifest::{InstallManifest, ManifestFile};
use crate::opencode::plugins::{install_loader, plugin_entry, skills_path, LOADER_REL_PATH, MANAGED_DIR};
use crate::source::cache::read_local_tree;
use crate::state::backups::backup_file;
use crate::state::state::State;
use crate::state::write_atomic;

/// Source-tree dirs ce-ai manages; the `.opencode/` prefix is stripped on copy.
const MANAGED_PREFIXES: [&str; 2] = [".opencode/plugins", ".opencode/skills"];

#[derive(clap::Args)]
pub struct Args {
    /// Harness to install into (v1: opencode only).
    #[arg(long)]
    pub harness: String,
    /// Local CE source tree; bypasses GitHub release fetching.
    #[arg(long)]
    pub source: PathBuf,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    if args.harness != "opencode" {
        return Err(CeError::Usage(format!(
            "unsupported harness '{}' — v1 supports opencode only",
            args.harness
        )));
    }
    let source = args.source.to_string_lossy().into_owned();
    // Desired managed files: managed-rel path -> (source-rel path, sha256).
    let managed: BTreeMap<String, (String, String)> = read_local_tree(&args.source)?
        .into_iter()
        .filter(|(rel, _)| MANAGED_PREFIXES.iter().any(|p| rel.starts_with(p)))
        .map(|(rel, hash)| {
            let managed_rel = rel.trim_start_matches(".opencode/").to_string();
            (managed_rel, (rel, hash))
        })
        .collect();
    if !managed.contains_key(LOADER_REL_PATH) {
        return Err(CeError::Runtime(format!(
            "CE loader not found at {}/.opencode/plugins/compound-engineering.js",
            args.source.display()
        )));
    }

    let config_dir = &ctx.opencode_config_dir;
    let opencode_json = config_dir.join("opencode.json");
    let managed_dir = config_dir.join(MANAGED_DIR);
    let needs_backup = opencode_json.exists();

    // Dry-run plans only; SU-4 guarantees zero writes.
    if ctx.dry_run {
        println!("plan: {}", if needs_backup { "backup opencode.json" } else { "create opencode.json" });
        for rel in managed.keys() {
            println!("plan: copy {rel}");
        }
        println!("plan: write install-manifest.json");
        println!("plan: update state.json");
        return Ok(());
    }

    // Apply: back up the existing config, then copy managed files (OI-1, OI-3).
    let backup = if needs_backup {
        Some(backup_file(&ctx.config_dir.join("backups"), &opencode_json)?)
    } else {
        None
    };
    let mut files = vec![install_loader(&args.source, config_dir)?];
    for (rel, (source_rel, hash)) in &managed {
        if rel == LOADER_REL_PATH {
            continue;
        }
        write_atomic(&managed_dir.join(rel), &std::fs::read(args.source.join(source_rel))?)?;
        files.push(ManifestFile { path: rel.clone(), sha256: hash.clone() });
    }

    // Merge plugin entry + skills path into opencode.json (OI-2, OI-4).
    let mut mutation = ensure_plugin_and_skills(
        &opencode_json,
        &plugin_entry(config_dir).to_string_lossy(),
        &skills_path(config_dir).to_string_lossy(),
    )?;
    mutation.backup = backup.map(|p| p.display().to_string());

    // Record managed files and the config mutation (OI-5).
    InstallManifest {
        version: "local".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: Utc::now().to_rfc3339(),
        source: serde_json::json!({ "kind": "local", "path": source }),
        files,
        config_mutations: vec![mutation],
    }
    .write(config_dir)?;

    // Update state.json; replace any prior opencode entry (idempotent).
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    state.installed_harnesses.retain(|h| h["name"].as_str() != Some("opencode"));
    state.installed_harnesses.push(serde_json::json!({
        "name": "opencode",
        "version": "local",
        "source": { "kind": "local", "path": source },
        "installed_at": Utc::now().to_rfc3339(),
        "last_synced_at": Utc::now().to_rfc3339(),
    }));
    state.save(&state_path)?;

    if !ctx.quiet {
        println!("installed compound-engineering for opencode (source: {source})");
    }
    Ok(())
}