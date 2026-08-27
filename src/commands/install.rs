//! `ce-ai install`: resolve source, plan, back up, then apply (OI-1..OI-5, SU-4).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::{ensure_plugin_and_skills, ConfigMutation};
use crate::opencode::manifest::{InstallManifest, ManifestFile};
use crate::opencode::plugins::{
    install_loader, plugin_entry, skills_path, LOADER_REL_PATH, MANAGED_DIR,
};
use crate::source::archive::extract_to_source;
use crate::source::cache::{managed_tree, record_tarball_provenance, Cache};
use crate::source::release::{
    github_token_from_env, pinned_version_and_url, resolve_latest_release,
};
use crate::state::backups::backup_file;
use crate::state::state::{ReleaseProvenance, State};
use crate::state::write_atomic;

/// Source-tree dirs ce-ai manages; see `crate::source::cache::MANAGED_PREFIXES`.

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
    /// Custom mode (--harness custom): target directory for CE plugin assets.
    #[arg(long)]
    pub plugins_dir: Option<PathBuf>,
    /// Custom mode (--harness custom): target directory for CE skill folders.
    #[arg(long)]
    pub skills_dir: Option<PathBuf>,
    /// Custom mode (--harness custom): markdown rules file receiving the
    /// managed CE block.
    #[arg(long)]
    pub rules_file: Option<PathBuf>,
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

    if target_harnesses.contains(&HarnessKind::Deepseek) {
        return Err(CeError::Usage(
            "deepseek harness is unsupported during developer preview (DeepSeek Harness 'dsh' uses YAML patch layers under ~/.dsh). Please use a supported native harness (opencode, claude, codex, copilot, cursor, grok, kimi, agy, pi, fx).".to_string()
        ));
    }

    let (source_path, version, source_json, tmp_dir) = resolve_source(ctx, &args.source)?;

    let managed: BTreeMap<String, (String, String)> = managed_tree(&source_path)?;
    if !managed.contains_key(LOADER_REL_PATH) {
        let err = Err(CeError::Runtime(format!(
            "CE loader not found at {}/.opencode/plugins/compound-engineering.js",
            source_path.display()
        )));
        if let Some(tmp) = tmp_dir {
            crate::state::report_best_effort_remove(&tmp, std::fs::remove_dir_all(&tmp));
        }
        return err;
    }

    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;

    // Transactional journal (#166): every tracked mutation records prior
    // content before being performed; state.json stays the final write.
    let mut journal = if ctx.dry_run {
        None
    } else {
        Some(crate::state::journal::Journal::begin(
            &ctx.config_dir,
            "install",
        )?)
    };
    macro_rules! arm {
        ($p:expr) => {
            if let Some(j) = journal.as_mut() {
                j.arm($p)?;
            }
        };
    }

    let home_dir = crate::harness::home_dir_from_ctx(ctx);

    for harness_kind in &target_harnesses {
        // R4: resolve custom-mode targets up front so dry-run planning and
        // apply share one validated configuration (Usage fast-fail included).
        let custom_cfg = if *harness_kind == HarnessKind::Custom {
            let flags = crate::harness::custom::CustomConfigFlags {
                plugins_dir: args.plugins_dir.clone(),
                skills_dir: args.skills_dir.clone(),
                rules_file: args.rules_file.clone(),
            };
            Some(crate::harness::custom::CustomHarnessConfig::resolve(
                &home_dir, &flags,
            )?)
        } else {
            None
        };
        let config_dir = if scope_arg == "workspace" {
            target_base_dir.clone()
        } else if *harness_kind == HarnessKind::Opencode {
            ctx.opencode_config_dir.clone()
        } else {
            harness_kind.harness_dir(&home_dir)
        };
        let target_config = harness_kind.config_path(&config_dir);
        let needs_backup = custom_cfg.is_none() && target_config.exists();

        // Dry-run plans only; SU-4 guarantees zero writes.
        if ctx.dry_run {
            if let Some(cfg) = &custom_cfg {
                println!("plan: create {}", cfg.plugins_dir.display());
                println!("plan: create {}", cfg.skills_dir.display());
                for rel in managed.keys() {
                    println!("plan: copy {rel}");
                }
                if let Some(rules) = &cfg.rules_file {
                    println!("plan: ensure managed CE block in {}", rules.display());
                }
                println!("plan: write install-manifest.json");
                println!("plan: update state.json");
                continue;
            }
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
        let managed_dir = config_dir.join(MANAGED_DIR);
        let mut files: Vec<ManifestFile> = Vec::new();
        if let Some(cfg) = &custom_cfg {
            // R4 layout: plugins/<rest> → <plugins_dir>/<rest>,
            // skills/<rest> → <skills_dir>/<rest>. No fabricated config file.
            std::fs::create_dir_all(&cfg.plugins_dir)?;
            std::fs::create_dir_all(&cfg.skills_dir)?;
            for (rel, (source_rel, hash)) in &managed {
                let dest = if let Some(rest) = crate::harness::custom::plugin_rel(rel) {
                    cfg.plugins_dir.join(rest)
                } else if let Some(rest) = crate::harness::custom::skill_rel(rel) {
                    cfg.skills_dir.join(rest)
                } else {
                    continue;
                };
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                arm!(&dest);
                write_atomic(&dest, &std::fs::read(source_path.join(source_rel))?)?;
                files.push(ManifestFile {
                    path: rel.clone(),
                    sha256: hash.clone(),
                });
            }
        } else {
            arm!(&plugin_entry(&config_dir));
            files.push(install_loader(&source_path, &config_dir)?);
            for (rel, (source_rel, hash)) in &managed {
                if rel == LOADER_REL_PATH {
                    continue;
                }
                arm!(&managed_dir.join(rel));
                write_atomic(
                    &managed_dir.join(rel),
                    &std::fs::read(source_path.join(source_rel))?,
                )?;
                files.push(ManifestFile {
                    path: rel.clone(),
                    sha256: hash.clone(),
                });
            }
        }

        // Write target config settings (OI-2) and install-manifest.json (SU-2).
        if *harness_kind == HarnessKind::Custom {
            let cfg = custom_cfg.as_ref().expect("custom config resolved above");
            // Route through the adapter so custom-mode behavior stays in one place.
            let adapter = crate::harness::custom::CustomAdapter::new(Some(cfg.clone()));

            let mut mutations: Vec<ConfigMutation> = Vec::new();
            if let Some(rules) = adapter.config().and_then(|c| c.rules_file.clone()) {
                let backup_id = if rules.exists() {
                    Some(
                        backup_file(&ctx.config_dir.join("backups"), &rules)?
                            .display()
                            .to_string(),
                    )
                } else {
                    None
                };
                arm!(&rules);
                if crate::harness::custom::ensure_rules_block(&rules)? && !ctx.quiet {
                    println!("install: managed CE block ensured in {}", rules.display());
                }
                mutations.push(ConfigMutation {
                    file: rules.display().to_string(),
                    backup: backup_id,
                    keys: vec!["ce-ai:block".into()],
                });
            }

            arm!(&cfg
                .plugins_dir
                .join(MANAGED_DIR)
                .join("install-manifest.json"));
            InstallManifest {
                version: version.to_string(),
                plugin_name: "compound-engineering".into(),
                installed_at: chrono::Utc::now().to_rfc3339(),
                source: source_json.clone(),
                files,
                config_mutations: mutations,
            }
            .write(&cfg.plugins_dir)?;
        } else if let Some(spec) = crate::harness::registration::registration_spec(*harness_kind) {
            // Strategy table: one exhaustive entry per table-driven kind
            // (see harness::registration). Skills are never copied into
            // harness-owned directories — adoption (skills adopt) is the only
            // delivery path (token-neutrality, R4).
            spec.register_companions(&target_config)?;
            arm!(&config_dir.join(MANAGED_DIR).join("install-manifest.json"));
            InstallManifest {
                version: version.to_string(),
                plugin_name: "compound-engineering".into(),
                installed_at: chrono::Utc::now().to_rfc3339(),
                source: source_json.clone(),
                files,
                config_mutations: vec![],
            }
            .write(&config_dir)?;
        } else {
            // The only kind reaching this arm is OpenCode itself: its native
            // registration is the plugin-entry + skills-paths JSON merge.
            arm!(&target_config);
            let mut mutation = ensure_plugin_and_skills(
                &target_config,
                &plugin_entry(&config_dir).to_string_lossy(),
                &skills_path(&config_dir).to_string_lossy(),
            )?;
            mutation.backup = backup.map(|p| p.display().to_string());

            InstallManifest {
                version: version.to_string(),
                plugin_name: "compound-engineering".into(),
                installed_at: chrono::Utc::now().to_rfc3339(),
                source: source_json.clone(),
                files,
                config_mutations: vec![mutation],
            }
            .write(&config_dir)?;
        }

        // Ensure the structural `ce-ai` orchestrator agent exists.
        arm!(&target_config);
        let agent_ensured = !ctx.dry_run
            && crate::harness::agents::ensure_orchestrator_agent(&target_config, harness_kind)?;
        if agent_ensured && !ctx.quiet {
            println!(
                "install: orchestrator agent '{}' ensured for {}",
                crate::harness::agents::ORCHESTRATOR_AGENT,
                harness_kind
            );
        }

        // Update state.json; replace any prior entry for this harness (idempotent).
        let harness_name = harness_kind.to_string();
        state
            .installed_harnesses
            .retain(|h| h["name"].as_str() != Some(harness_name.as_str()));
        let mut entry = serde_json::json!({
            "name": harness_name,
            "version": version,
            "source": source_json,
            "installed_at": Utc::now().to_rfc3339(),
            "last_synced_at": Utc::now().to_rfc3339(),
        });
        if let Some(cfg) = &custom_cfg {
            entry["custom"] = cfg.to_state_json();
        }
        state.installed_harnesses.push(entry);

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
        if let Some(j) = journal.take() {
            j.complete()?;
        }
        if let Err(e) = crate::source::registry::SkillRegistry::sync_registry(ctx) {
            if !ctx.quiet {
                eprintln!("warning: skill registry sync failed: {e}");
            }
        }
    }

    if let Some(tmp) = tmp_dir {
        crate::state::report_best_effort_remove(&tmp, std::fs::remove_dir_all(&tmp));
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

    // Default: fetch latest release from GitHub — never a mutable branch.
    let client = reqwest::blocking::Client::new();
    let token = github_token_from_env();
    let tag = resolve_latest_release(&client, token.as_deref())?;
    let (version, url) = pinned_version_and_url(tag)?;
    let bytes = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|err| CeError::Network(format!("release download failed: {err}")))?
        .bytes()
        .map_err(|err| CeError::Runtime(err.to_string()))?;
    let (tarball, hex, _dry_run_tmp) = if ctx.dry_run {
        let tmp = tempfile::TempDir::new()?;
        let tarball_path = tmp.path().join("dry_run.tar.gz");
        std::fs::write(&tarball_path, &bytes)?;
        use sha2::Digest;
        let hex = format!("{:x}", sha2::Sha256::digest(&bytes));
        (tarball_path, hex, Some(tmp))
    } else {
        let (tarball, hex) = Cache::new(ctx.config_dir.join("cache")).cache_tarball(&bytes)?;
        (tarball, hex, None)
    };
    let (root, tmp) = extract_to_source(&ctx.config_dir, ctx.dry_run, &tarball, &version)?;
    if !ctx.dry_run {
        record_tarball_provenance(
            &ctx.config_dir.join("state.json"),
            ReleaseProvenance {
                tag: version.clone(),
                url,
                archive_sha256: hex,
                extraction_path: root.clone(),
            },
        )?;
    }
    let source_json = serde_json::json!({ "kind": "github-release", "tag": version, "tree": root });
    Ok((root, version, source_json, tmp))
}

#[cfg(test)]
mod tests {
    use crate::state::{ConfigStore, InMemoryConfigStore, InMemoryStateStore, StateStore};
    use std::path::Path;

    #[test]
    fn install_state_store_port_loads_and_saves_without_filesystem() {
        let store = InMemoryStateStore::new();
        let path = Path::new("/virtual/ce-ai/state.json");

        let mut state = store.load(path).unwrap();
        assert_eq!(state.version, 1);
        state.installed_harnesses.push(serde_json::json!({
            "name": "opencode",
            "version": "1.0.0",
            "installed_at": "2026-08-27T00:00:00Z"
        }));
        store.save(path, &state).unwrap();

        let loaded = store.load(path).unwrap();
        assert_eq!(loaded.installed_harnesses.len(), 1);
        assert_eq!(loaded.installed_harnesses[0]["name"], "opencode");
    }

    #[test]
    fn install_config_store_port_mutates_without_filesystem() {
        let store = InMemoryConfigStore::new();
        let path = Path::new("/virtual/config/opencode.json");

        let mutation = crate::opencode::config::ensure_plugin_and_skills_with_store(
            &store,
            path,
            "/virtual/plugin.js",
            "/virtual/skills",
        )
        .unwrap();

        assert_eq!(mutation.keys, vec!["plugin", "skills.paths"]);
        let cfg = store.read_config(path).unwrap();
        assert_eq!(cfg["plugin"], serde_json::json!(["/virtual/plugin.js"]));
    }
}
