//! ce-ai: compound-engineering plugin manager CLI entry point (CC-1, CC-2).

#![forbid(unsafe_code)]

mod commands;
mod error;
mod harness;
mod opencode;
mod source;
mod state;
mod tui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::{
    backups, deinit_prj, doctor, init_prj, install, models, status, sync, tools, uninstall,
    upgrade, workflow, Context,
};
use crate::error::result_exit_code;

#[derive(Parser)]
#[command(name = "ce-ai", about = "compound-engineering plugin manager", version)]
struct Cli {
    /// ce-ai data dir (state.json, backups, cache); defaults to ~/.ce-ai.
    #[arg(long, global = true)]
    config_dir: Option<PathBuf>,
    /// Preview planned changes without writing (SU-4).
    #[arg(long, global = true)]
    dry_run: bool,
    /// Verbose output.
    #[arg(short = 'v', long, global = true)]
    verbose: bool,
    /// Quiet output; suppress non-error messages.
    #[arg(short = 'q', long, global = true)]
    quiet: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install the CE plugin into a harness.
    Install(install::Args),
    /// Reconcile the installed plugin against the current source tree.
    Sync(sync::Args),
    /// Fetch a newer CE source and sync the installed plugin.
    Upgrade(upgrade::Args),
    /// Manage model assignments and named profiles.
    Models(models::Args),
    /// Show installed harnesses, versions, and drift.
    Status,
    /// Remove the CE plugin and restore the pre-install config.
    Uninstall(uninstall::Args),
    /// Report config validity, drift, and state consistency.
    Doctor,
    /// Backup listing and point-in-time config recovery.
    Backups(backups::BackupsArgs),
    /// Companion developer sidecars and memory tools manager (Engram, CodeGraph, Context7, RTK).
    Tools(tools::Args),
    /// Workflow FSM & progress recovery system across 7 development stages.
    Workflow(workflow::Args),
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
    },
    /// Remove managed Compound Engineering workflow blocks from a project repository cleanly.
    #[command(name = "deinit-prj")]
    DeinitPrj {
        /// Target project directory path (default: current working directory)
        path: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let ctx = match Context::resolve(cli.config_dir, cli.dry_run, cli.verbose, cli.quiet) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(err.exit_code());
        }
    };
    let result = match cli.command {
        Some(Commands::Install(args)) => install::run(&ctx, &args),
        Some(Commands::Sync(args)) => sync::run(&ctx, &args),
        Some(Commands::Upgrade(args)) => upgrade::run(&ctx, &args),
        Some(Commands::Models(args)) => models::run(&ctx, &args),
        Some(Commands::Status) => status::run(&ctx),
        Some(Commands::Uninstall(args)) => uninstall::run(&ctx, &args),
        Some(Commands::Doctor) => doctor::run(&ctx),
        Some(Commands::Backups(args)) => backups::run(&ctx, &args),
        Some(Commands::Tools(args)) => tools::run(&ctx, &args),
        Some(Commands::Workflow(args)) => workflow::run(&ctx, &args),
        Some(Commands::InitPrj { path, tier, force }) => init_prj::run(&ctx, path, &tier, force),
        Some(Commands::DeinitPrj { path }) => deinit_prj::run(&ctx, path),
        None => tui::run_interactive(&ctx),
    };
    if let Err(err) = &result {
        eprintln!("error: {err}");
    }
    std::process::exit(result_exit_code(&result));
}
