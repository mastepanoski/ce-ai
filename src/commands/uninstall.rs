//! `ce-ai uninstall`: restore pre-install config and optionally remove all managed assets (CC-3).

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
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
}

impl Default for Args {
    fn default() -> Self {
        Self {
            harness: "opencode".into(),
            all: false,
            yes: true,
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
            } else if target_config.exists() {
                std::fs::remove_file(&target_config)?;
            }
            let managed_dir = config_dir.join(MANAGED_DIR);
            if managed_dir.exists() {
                std::fs::remove_dir_all(&managed_dir)?;
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
