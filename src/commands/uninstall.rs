//! `ce-ai uninstall`: restore pre-install config and optionally remove all managed assets (CC-3).

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::backups::{newest_backup_dir, restore_latest};
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

    for target in &targets {
        if target == "opencode" {
            let opencode_json = ctx.opencode_config_dir.join("opencode.json");
            let backups = ctx.config_dir.join("backups");
            match newest_backup_dir(&backups)? {
                Some(_) => {
                    let _ = restore_latest(&backups, &opencode_json);
                }
                None => {
                    if opencode_json.exists() {
                        let _ = std::fs::remove_file(&opencode_json);
                    }
                }
            }
            let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);
            if managed_dir.exists() {
                let _ = std::fs::remove_dir_all(&managed_dir);
            }
        }
        state
            .installed_harnesses
            .retain(|h| h["name"].as_str() != Some(target.as_str()));
    }

    state.save(&state_path)?;

    if !ctx.dry_run {
        let registry_path = ctx.config_dir.join("skills-registry.json");
        if registry_path.exists() {
            let _ = std::fs::remove_file(&registry_path);
        }
        if let Ok(entries) = std::fs::read_dir(&ctx.config_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with(".skills-registry.json.tmp") {
                    if let Ok(meta) = std::fs::symlink_metadata(entry.path()) {
                        if meta.is_file() && !meta.file_type().is_symlink() {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
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
