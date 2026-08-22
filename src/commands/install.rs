//! `ce-ai install`: resolve source, plan, back up, then apply (OI-1..OI-5, SU-4).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::ensure_plugin_and_skills;
use crate::opencode::manifest::{InstallManifest, ManifestFile};
use crate::opencode::plugins::{
    install_loader, plugin_entry, skills_path, LOADER_REL_PATH, MANAGED_DIR,
};
use crate::source::archive::extract_to_source;
use crate::source::cache::{read_local_tree, Cache};
use crate::source::release::{
    github_token_from_env, main_tarball_url, resolve_latest_release, tag_tarball_url,
};
use crate::state::backups::backup_file;
use crate::state::state::State;
use crate::state::write_atomic;

/// Source-tree dirs ce-ai manages; the `.opencode/` prefix is stripped on copy.
const MANAGED_PREFIXES: [&str; 2] = [".opencode/plugins", ".opencode/skills"];

#[derive(clap::Args)]
pub struct Args {
    /// Harness to install into (e.g. opencode, claude, or all).
    #[arg(long)]
    pub harness: String,
    /// Local CE source tree; bypasses GitHub release fetching.
    #[arg(long)]
    pub source: Option<PathBuf>,
    /// Installation scope: global (default) or workspace (repository root).
    #[arg(long, default_value = "global")]
    pub scope: String,
}

use crate::harness::HarnessKind;

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let harness_arg = args.harness.to_lowercase();
    let scope_arg = args.scope.to_lowercase();

    let target_base_dir = if scope_arg == "workspace" {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let repo_root = String::from_utf8_lossy(&out.stdout).trim().to_string();
                PathBuf::from(repo_root)
            } else {
                ctx.opencode_config_dir.clone()
            }
        } else {
            ctx.opencode_config_dir.clone()
        }
    } else {
        ctx.opencode_config_dir.clone()
    };
    let target_harnesses: Vec<HarnessKind> = if harness_arg == "all" {
        if let Ok(home) = std::env::var("HOME") {
            let detected = HarnessKind::detect_installed_harnesses(Path::new(&home));
            if detected.is_empty() {
                vec![HarnessKind::Opencode]
            } else {
                detected
            }
        } else {
            vec![HarnessKind::Opencode]
        }
    } else {
        vec![harness_arg.parse::<HarnessKind>()?]
    };

    let (source_path, version, source_json, tmp_dir) = resolve_source(ctx, &args.source)?;

    let managed: BTreeMap<String, (String, String)> = read_local_tree(&source_path)?
        .into_iter()
        .filter(|(rel, _)| MANAGED_PREFIXES.iter().any(|p| rel.starts_with(p)))
        .map(|(rel, hash)| {
            let managed_rel = rel.trim_start_matches(".opencode/").to_string();
            (managed_rel, (rel, hash))
        })
        .collect();
    if !managed.contains_key(LOADER_REL_PATH) {
        let err = Err(CeError::Runtime(format!(
            "CE loader not found at {}/.opencode/plugins/compound-engineering.js",
            source_path.display()
        )));
        if let Some(tmp) = tmp_dir {
            let _ = std::fs::remove_dir_all(tmp);
        }
        return err;
    }

    let config_dir = &target_base_dir;
    let managed_dir = config_dir.join(MANAGED_DIR);

    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;

    for harness_kind in &target_harnesses {
        let target_config = harness_kind.config_path(config_dir);
        let needs_backup = target_config.exists();

        // Dry-run plans only; SU-4 guarantees zero writes.
        if ctx.dry_run {
            println!(
                "plan: {}",
                if needs_backup {
                    format!("backup {}", target_config.display())
                } else {
                    format!("create {}", target_config.display())
                }
            );
            for rel in managed.keys() {
                println!("plan: copy {rel}");
            }
            println!("plan: write install-manifest.json");
            println!("plan: update state.json");
            continue;
        }

        // Apply: back up the existing config, then copy managed files (OI-1, OI-3).
        let backup = if needs_backup {
            Some(backup_file(
                &ctx.config_dir.join("backups"),
                &target_config,
            )?)
        } else {
            None
        };
        let mut files = vec![install_loader(&source_path, config_dir)?];
        for (rel, (source_rel, hash)) in &managed {
            if rel == LOADER_REL_PATH {
                continue;
            }
            write_atomic(
                &managed_dir.join(rel),
                &std::fs::read(source_path.join(source_rel))?,
            )?;
            files.push(ManifestFile {
                path: rel.clone(),
                sha256: hash.clone(),
            });
        }

        // Merge plugin entry + skills path into target harness config (OI-2, OI-4).
        let mut mutation = ensure_plugin_and_skills(
            &target_config,
            &plugin_entry(config_dir).to_string_lossy(),
            &skills_path(config_dir).to_string_lossy(),
        )?;
        mutation.backup = backup.map(|p| p.display().to_string());

        // Record managed files and the config mutation (OI-5).
        InstallManifest {
            version: version.clone(),
            plugin_name: "compound-engineering".into(),
            installed_at: Utc::now().to_rfc3339(),
            source: source_json.clone(),
            files,
            config_mutations: vec![mutation],
        }
        .write(config_dir)?;

        // Update state.json; replace any prior entry for this harness (idempotent).
        let harness_name = harness_kind.to_string();
        state
            .installed_harnesses
            .retain(|h| h["name"].as_str() != Some(harness_name.as_str()));
        state.installed_harnesses.push(serde_json::json!({
            "name": harness_name,
            "version": version,
            "source": source_json,
            "installed_at": Utc::now().to_rfc3339(),
            "last_synced_at": Utc::now().to_rfc3339(),
        }));

        if !ctx.quiet && !ctx.dry_run {
            let source_disp = if args.source.is_some() {
                source_path.display().to_string()
            } else {
                source_json["tag"].as_str().unwrap_or("release").to_string()
            };
            println!("installed compound-engineering for {harness_name} (source: {source_disp})");
        }
    }

    if !ctx.dry_run {
        state.save(&state_path)?;
        // Seed documented default model assignments (incl. orchestrator slot
        // `ce-ai`) into slots the user has not configured (#111).
        for (slot, model) in crate::commands::models::apply_defaults(ctx)? {
            if !ctx.quiet {
                println!("install: default model {slot} = {model}");
            }
        }
    }

    if let Some(tmp) = tmp_dir {
        let _ = std::fs::remove_dir_all(tmp);
    }
    Ok(())
}

fn resolve_source(
    ctx: &Context,
    source_arg: &Option<PathBuf>,
) -> Result<(PathBuf, String, serde_json::Value, Option<PathBuf>), CeError> {
    if let Some(path) = source_arg {
        let version = "local".to_string();
        let source_json = serde_json::json!({ "kind": "local", "path": path });
        return Ok((path.clone(), version, source_json, None));
    }

    // Try using cached tarball if present in state
    let state_path = ctx.config_dir.join("state.json");
    if state_path.exists() {
        if let Ok(state) = State::load(&state_path) {
            if let Some(digest) = state.managed_asset_digest.get("tarball") {
                if let Some(hex) = digest.strip_prefix("sha256:") {
                    let cached_tarball = ctx
                        .config_dir
                        .join("cache")
                        .join(format!("ce-{hex}.tar.gz"));
                    if cached_tarball.exists() {
                        let (root, tmp) = extract_to_source(
                            &ctx.config_dir,
                            ctx.dry_run,
                            &cached_tarball,
                            "cached",
                        )?;
                        let source_json = serde_json::json!({ "kind": "github-release", "tag": "cached", "tree": root });
                        return Ok((root, "cached".to_string(), source_json, tmp));
                    }
                }
            }
        }
    }

    // Default: fetch latest release from GitHub
    let client = reqwest::blocking::Client::new();
    let token = github_token_from_env();
    let tag = resolve_latest_release(&client, token.as_deref())?;
    let (version, url) = match tag {
        Some(tag) => (tag.clone(), tag_tarball_url(&tag)),
        None => ("main".to_string(), main_tarball_url()),
    };
    let bytes = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|err| CeError::Runtime(format!("release download failed: {err}")))?
        .bytes()
        .map_err(|err| CeError::Runtime(err.to_string()))?;
    let tarball = Cache::new(ctx.config_dir.join("cache"))
        .cache_tarball(&bytes, &ctx.config_dir.join("state.json"))?;
    let (root, tmp) = extract_to_source(&ctx.config_dir, ctx.dry_run, &tarball, &version)?;
    let source_json = serde_json::json!({ "kind": "github-release", "tag": version, "tree": root });
    Ok((root, version, source_json, tmp))
}
