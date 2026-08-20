//! CLI subcommands: install, sync, upgrade, models, status, uninstall, doctor.

pub mod doctor;
pub mod install;
pub mod models;
pub mod status;
pub mod sync;
pub mod uninstall;
pub mod upgrade;

use std::path::PathBuf;

use crate::error::CeError;

/// Shared command context: resolved dirs + global flag state (CC-2).
pub struct Context {
    pub config_dir: PathBuf,
    pub opencode_config_dir: PathBuf,
    pub dry_run: bool,
    pub verbose: bool,
    pub quiet: bool,
}

impl Context {
    /// Resolves the ce-ai data dir and the OpenCode config dir from global
    /// flags and env; temp dirs in tests keep every run hermetic.
    pub fn resolve(
        config_dir: Option<PathBuf>,
        dry_run: bool,
        verbose: bool,
        quiet: bool,
    ) -> Result<Self, CeError> {
        let config_dir = match config_dir {
            Some(dir) => dir,
            None => {
                let home = std::env::var("HOME")
                    .map_err(|_| CeError::Usage("cannot resolve HOME; pass --config-dir".into()))?;
                PathBuf::from(home).join(".ce-ai")
            }
        };
        let opencode_config_dir = match std::env::var("CE_AI_OPENCODE_CONFIG") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => {
                let home = std::env::var("HOME").unwrap_or_default();
                PathBuf::from(home).join(".config/opencode")
            }
        };
        Ok(Self {
            config_dir,
            opencode_config_dir,
            dry_run,
            verbose,
            quiet,
        })
    }
}
