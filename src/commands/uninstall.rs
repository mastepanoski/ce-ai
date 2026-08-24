//! `ce-ai uninstall`: restore pre-install config and optionally remove all managed assets (CC-3).

use std::path::PathBuf;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::custom::{
    plugin_rel, prune_empty_dirs, skill_rel, strip_rules_block, CustomConfigFlags,
    CustomHarnessConfig,
};
use crate::harness::HarnessKind;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::state::State;

#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    /// Harness to uninstall (e.g. opencode, claude, cursor, or 'all').
    #[arg(long, default_value = "opencode")]
    pub harness: String,

    /// Complete removal of all managed loader scripts and skills directories.
    #[arg(long, default_value_t = false)]
    pub all: bool,

    /// Bypass interactive confirmation prompt.
    #[arg(short = 'y', long, default_value_t = false)]
    pub yes: bool,

    /// Custom mode (--harness custom): directory holding CE plugin assets.
    #[arg(long)]
    pub plugins_dir: Option<PathBuf>,
    /// Custom mode (--harness custom): directory holding CE skill folders.
    #[arg(long)]
    pub skills_dir: Option<PathBuf>,
    /// Custom mode (--harness custom): markdown rules file carrying the
    /// managed CE block.
    #[arg(long)]
    pub rules_file: Option<PathBuf>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            harness: "opencode".into(),
            all: false,
            yes: true,
            plugins_dir: None,
            skills_dir: None,
            rules_file: None,
        }
    }
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let targets: Vec<String> = if args.harness == "all" {
        HarnessKind::all_str()
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        let _ = args.harness.parse::<HarnessKind>()?;
        vec![args.harness.clone()]
    };

    if args.all && !args.yes && !ctx.quiet {
        println!(
            "⚠️ Notice: Performing complete removal of managed assets for targets: {:?}",
            targets
        );
    }

    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;

    let home_dir = crate::harness::home_dir_from_ctx(ctx);

    for target in &targets {
        if let Ok(harness_kind) = target.parse::<HarnessKind>() {
            if harness_kind == HarnessKind::Deepseek {
                return Err(CeError::Usage(
                    "deepseek harness is unsupported during developer preview (DeepSeek Harness 'dsh' uses YAML patch layers under ~/.dsh). Please use a supported native harness (opencode, claude, codex, copilot, cursor, grok, kimi, agy, pi, fx).".to_string()
                ));
            }
            if harness_kind == HarnessKind::Custom {
                // R4 surgical removal: state snapshot ▸ flags ▸ config file.
                let snapshot = state
                    .installed_harnesses
                    .iter()
                    .find(|h| h["name"].as_str() == Some("custom"))
                    .and_then(|h| CustomHarnessConfig::from_state_json(&h["custom"]));
                let cfg = match snapshot {
                    Some(cfg) => cfg,
                    None => {
                        match CustomHarnessConfig::resolve(
                            &home_dir,
                            &CustomConfigFlags {
                                plugins_dir: args.plugins_dir.clone(),
                                skills_dir: args.skills_dir.clone(),
                                rules_file: args.rules_file.clone(),
                            },
                        ) {
                            Ok(cfg) => cfg,
                            Err(err @ CeError::Usage(_)) => {
                                // `--all` must not abort just because no
                                // custom install exists on this host.
                                if targets.len() > 1 {
                                    continue;
                                }
                                return Err(err);
                            }
                            Err(err) => return Err(err),
                        }
                    }
                };

                match InstallManifest::load(&cfg.plugins_dir) {
                    Ok(manifest) => {
                        for file in &manifest.files {
                            let dest = if let Some(rest) = plugin_rel(&file.path) {
                                cfg.plugins_dir.join(rest)
                            } else if let Some(rest) = skill_rel(&file.path) {
                                cfg.skills_dir.join(rest)
                            } else {
                                continue;
                            };
                            if dest.is_file() {
                                std::fs::remove_file(&dest)?;
                            }
                            if let Some(parent) = dest.parent() {
                                prune_empty_dirs(parent, &[&cfg.plugins_dir, &cfg.skills_dir]);
                            }
                        }
                        let manifest_path = cfg
                            .plugins_dir
                            .join(MANAGED_DIR)
                            .join("install-manifest.json");
                        if manifest_path.is_file() {
                            std::fs::remove_file(&manifest_path)?;
                        }
                    }
                    Err(_) => {
                        if !ctx.quiet {
                            eprintln!(
                                "warning: no install manifest under {}; nothing surgical to remove",
                                cfg.plugins_dir.display()
                            );
                        }
                    }
                }

                // Remove CE-created roots when they ended up empty; never
                // delete user-owned directories that still hold content.
                let _ = std::fs::remove_dir(cfg.plugins_dir.join(MANAGED_DIR));
                let _ = std::fs::remove_dir(&cfg.plugins_dir);
                let _ = std::fs::remove_dir(&cfg.skills_dir);

                if let Some(rules) = &cfg.rules_file {
                    if strip_rules_block(rules)? && !ctx.quiet {
                        println!(
                            "uninstall: managed CE block stripped from {}",
                            rules.display()
                        );
                    }
                }

                state
                    .installed_harnesses
                    .retain(|h| h["name"].as_str() != Some(target.as_str()));
                state.save(&state_path)?;
                continue;
            }
            let config_dir = if harness_kind == HarnessKind::Opencode {
                ctx.opencode_config_dir.clone()
            } else {
                harness_kind.harness_dir(&home_dir)
            };
            let target_config = harness_kind.config_path(&config_dir);
            let backups = ctx.config_dir.join("backups");
            if let Some(backup) =
                crate::state::backups::newest_backup_for_harness(&backups, target)?
            {
                crate::state::backups::restore_backup_by_id(&backups, &backup.id, &target_config)?;
            } else if harness_kind == HarnessKind::Cursor {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::cursor::unregister_cursor_mcp_server(&target_config, tool)?;
                }
            } else if harness_kind == HarnessKind::Kimi {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::kimi::unregister_kimi_mcp_server(&target_config, tool)?;
                }
            } else if harness_kind == HarnessKind::Agy {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::agy::unregister_agy_mcp_server(&target_config, tool)?;
                }
                let legacy_json = config_dir.join("antigravity-cli").join("antigravity.json");
                if legacy_json.exists() {
                    let _ = std::fs::remove_file(&legacy_json);
                }
            } else if harness_kind == HarnessKind::Claude {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::claude::unregister_claude_mcp_server(&target_config, tool)?;
                }
            } else if harness_kind == HarnessKind::Grok {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::grok::unregister_grok_mcp_server(&target_config, tool)?;
                }
            } else if harness_kind == HarnessKind::Copilot {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::copilot::unregister_copilot_mcp_server(&target_config, tool)?;
                }
            } else if harness_kind == HarnessKind::Codex {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::codex::unregister_codex_mcp_server(&target_config, tool)?;
                }
            } else if harness_kind == HarnessKind::Fx {
                for tool in &["codegraph", "engram", "context7", "rtk"] {
                    crate::harness::fx::unregister_fx_mcp_server(&target_config, tool)?;
                }
            } else if target_config.is_file() {
                std::fs::remove_file(&target_config)?;
            }
            let managed_dir = config_dir.join(MANAGED_DIR);
            if managed_dir.exists() {
                std::fs::remove_dir_all(&managed_dir)?;
            }
            if harness_kind == HarnessKind::Claude
                || harness_kind == HarnessKind::Codex
                || harness_kind == HarnessKind::Copilot
                || harness_kind == HarnessKind::Grok
                || harness_kind == HarnessKind::Kimi
                || harness_kind == HarnessKind::Agy
                || harness_kind == HarnessKind::Pi
                || harness_kind == HarnessKind::Fx
            {
                let skills_dir = if harness_kind == HarnessKind::Agy {
                    config_dir.join("config").join("skills")
                } else {
                    config_dir.join("skills")
                };
                if skills_dir.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&skills_dir) {
                        if !ctx.quiet {
                            eprintln!(
                                "warning: failed to clean skills directory at {}: {e}",
                                skills_dir.display()
                            );
                        }
                    }
                }
            }
        }
        state
            .installed_harnesses
            .retain(|h| h["name"].as_str() != Some(target.as_str()));
        state.save(&state_path)?;
    }

    if let Err(e) = crate::source::registry::SkillRegistry::remove(ctx) {
        if !ctx.quiet {
            eprintln!("warning: skill registry cleanup failed: {e}");
        }
    }

    if !ctx.quiet {
        if args.harness == "all" {
            println!("✅ Uninstalled all target harnesses cleanly.");
        } else {
            println!("✅ Uninstalled {} cleanly.", args.harness);
        }
    }
    Ok(())
}
