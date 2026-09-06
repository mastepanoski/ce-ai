//! Command registry — Strategy extraction for CLI dispatch (KTD1, R3).
//!
//! Centralizes the `Commands` enum and its `run` dispatch so `src/main.rs`
//! stays a thin entry point. Adding a new command touches this module and its
//! handler, not `main.rs`.

use std::path::PathBuf;

use clap::Subcommand;

use crate::commands::{
    audit, backups, deinit_prj, doctor, guard, init_prj, install, models, skills, status, sync,
    tools, uninstall, upgrade, usage, workflow, Context,
};
use crate::error::CeError;

/// Strategy interface for CLI commands (KTD1).
pub trait CeCommand {
    fn run(&self, ctx: &Context) -> Result<(), CeError>;
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install the CE plugin into a harness.
    Install(install::Args),
    /// Reconcile the installed plugin against the current source tree.
    Sync(sync::Args),
    /// Fetch a newer CE source and sync the installed plugin.
    Upgrade(upgrade::Args),
    /// Manage model assignments and named profiles.
    Models(models::Args),
    /// Multi-harness skill registry discovery, prompt resolution, and health diagnostics.
    Skills(skills::Args),
    /// Show installed harnesses, versions, and drift.
    Status,
    /// Remove the CE plugin and restore the pre-install config.
    Uninstall(uninstall::Args),
    /// Report config validity, drift, and state consistency.
    Doctor(doctor::Args),
    /// Backup listing and point-in-time config recovery.
    Backups(backups::BackupsArgs),
    /// Companion developer sidecars and memory tools manager (Engram, CodeGraph, Context7, RTK).
    Tools(tools::Args),
    /// Usage analytics: token capture and reporting.
    Usage(usage::Args),
    /// Workflow FSM & progress recovery system across 7 development stages.
    Workflow(workflow::Args),
    /// Multi-harness token-efficiency and context-quality audit engine.
    Audit(audit::Args),
    /// Adopt a project repository by injecting managed Compound Engineering workflow blocks into AGENTS.md.
    #[command(name = "init-prj")]
    InitPrj {
        /// Target project directory path (default: current working directory)
        path: Option<PathBuf>,
        /// Adoption tier: full, minimal, orchestrator
        #[arg(long, default_value = "full")]
        tier: String,
        /// Force overwrite of modified managed blocks
        #[arg(long)]
        force: bool,
        /// Skip auto-configuring RTK hook injection.
        #[arg(long, default_value_t = false)]
        skip_rtk: bool,
        /// Skip configuring all companion tools (both MCP and hooks).
        #[arg(long, default_value_t = false)]
        skip_companions: bool,
    },
    /// Remove managed Compound Engineering workflow blocks from a project repository cleanly.
    #[command(name = "deinit-prj")]
    DeinitPrj {
        /// Target project directory path (default: current working directory)
        path: Option<PathBuf>,
    },
    /// Pedagogical Guardrail Mode for junior developer oversight (Issue #114).
    Guard(guard::Args),
}

impl CeCommand for Commands {
    fn run(&self, ctx: &Context) -> Result<(), CeError> {
        match self {
            Commands::Install(args) => install::run(ctx, args),
            Commands::Sync(args) => sync::run(ctx, args),
            Commands::Upgrade(args) => upgrade::run(ctx, args),
            Commands::Models(args) => models::run(ctx, args),
            Commands::Skills(args) => skills::run(ctx, args),
            Commands::Status => status::run(ctx),
            Commands::Uninstall(args) => uninstall::run(ctx, args),
            Commands::Doctor(args) => doctor::run(ctx, args),
            Commands::Backups(args) => backups::run(ctx, args),
            Commands::Tools(args) => tools::run(ctx, args),
            Commands::Usage(sub) => crate::commands::usage::run(ctx, sub),
            Commands::Workflow(args) => workflow::run(ctx, args),
            Commands::Audit(args) => audit::run(ctx, args),
            Commands::InitPrj {
                path,
                tier,
                force,
                skip_rtk,
                skip_companions,
            } => init_prj::run(ctx, path.clone(), tier, *force, *skip_rtk, *skip_companions),
            Commands::DeinitPrj { path } => deinit_prj::run(ctx, path.clone()),
            Commands::Guard(args) => guard::run(ctx, args),
        }
    }
}

/// Registry dispatch — thin wrapper used by `main.rs` and `tui`.
/// Keeps `main.rs` at ~15 lines (KTD1).
pub fn dispatch(ctx: &Context, command: Option<Commands>) -> Result<(), CeError> {
    match command {
        Some(cmd) => cmd.run(ctx),
        None => crate::tui::run_interactive(ctx),
    }
}
