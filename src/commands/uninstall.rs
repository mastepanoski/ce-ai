//! `ce-ai uninstall`: restore the newest backup and remove managed files (CC-3).

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::backups::{newest_backup_dir, restore_latest};
use crate::state::state::State;

#[derive(clap::Args)]
pub struct Args {
    /// Harness to uninstall (v1: opencode only).
    #[arg(long)]
    pub harness: String,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    if args.harness != "opencode" {
        return Err(CeError::Usage(format!(
            "unsupported harness '{}' — v1 supports opencode only",
            args.harness
        )));
    }
    // Manifest presence marks an install; refuse to uninstall what isn't installed.
    let manifest = InstallManifest::load(&ctx.opencode_config_dir)
        .map_err(|_| CeError::Runtime("no install-manifest.json — nothing to uninstall".into()))?;

    // Restore the newest pre-install backup of opencode.json, or remove the
    // config file we created when no backup exists (CC-3).
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    let backups = ctx.config_dir.join("backups");
    match newest_backup_dir(&backups)? {
        Some(_) => restore_latest(&backups, &opencode_json)?,
        None => {
            if opencode_json.exists() {
                std::fs::remove_file(&opencode_json)?;
            }
        }
    }
    let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);
    if managed_dir.exists() {
        std::fs::remove_dir_all(&managed_dir)?;
    }

    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    state.installed_harnesses.retain(|h| h["name"].as_str() != Some(args.harness.as_str()));
    state.save(&state_path)?;

    if !ctx.quiet {
        println!("uninstalled {} ({})", args.harness, manifest.version);
    }
    Ok(())
}