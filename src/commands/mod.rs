//! CLI subcommands: install, sync, upgrade, models, status, uninstall, doctor.

pub mod adopt;
pub mod audit;
pub mod backups;
pub mod deinit_prj;
pub mod doctor;
pub mod guard;
pub mod init_prj;
pub mod install;
pub mod models;
pub mod registry;
pub mod skills;
pub mod status;
pub mod sync;
pub mod tools;
pub mod uninstall;
pub mod upgrade;
pub mod usage;
pub mod workflow;

use std::path::PathBuf;

use crate::error::CeError;

/// Shared command context: resolved dirs + global flag state (CC-2).
#[derive(Clone)]
pub struct Context {
    pub config_dir: PathBuf,
    pub opencode_config_dir: PathBuf,
    /// Repository root when the CLI runs inside a git work tree; enables
    /// `.ce-ai.json` workspace overrides of model assignments (MM-1).
    pub workspace_root: Option<std::path::PathBuf>,
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
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .map_err(|_| {
                        CeError::Usage(
                            "cannot resolve home directory (HOME/USERPROFILE not set); pass --config-dir"
                                .into(),
                        )
                    })?;
                PathBuf::from(home).join(".ce-ai")
            }
        };
        let opencode_config_dir = match std::env::var("CE_AI_OPENCODE_CONFIG") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_default();
                PathBuf::from(home).join(".config/opencode")
            }
        };
        let workspace_root = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()));

        Ok(Self {
            config_dir,
            opencode_config_dir,
            workspace_root,
            dry_run,
            verbose,
            quiet,
        })
    }

    /// Returns the canonical path to `state.json`.
    pub fn state_path(&self) -> PathBuf {
        self.config_dir.join("state.json")
    }

    /// Returns the canonical path to `opencode.json`.
    pub fn opencode_config_path(&self) -> PathBuf {
        self.opencode_config_dir.join("opencode.json")
    }

    /// Returns a `StateStore` instance for state I/O.
    pub fn state_store(&self) -> Box<dyn crate::state::StateStore> {
        Box::new(crate::state::FsStateStore)
    }

    /// Returns a `ConfigStore` instance for config I/O.
    pub fn config_store(&self) -> Box<dyn crate::state::ConfigStore> {
        Box::new(crate::state::FsConfigStore)
    }
}
