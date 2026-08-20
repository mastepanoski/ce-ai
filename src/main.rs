//! ce-ai: compound-engineering plugin manager CLI entry point (CC-1, CC-2).

mod commands;
mod error;
mod opencode;
mod source;
mod state;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands::{doctor, install, models, status, sync, uninstall, upgrade, Context};
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
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install the CE plugin into a harness.
    Install(install::Args),
    /// Reconcile the installed plugin against the current source tree.
    Sync,
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
        Commands::Install(args) => install::run(&ctx, &args),
        Commands::Sync => sync::run(&ctx),
        Commands::Upgrade(args) => upgrade::run(&ctx, &args),
        Commands::Models(args) => models::run(&ctx, &args),
        Commands::Status => status::run(&ctx),
        Commands::Uninstall(args) => uninstall::run(&ctx, &args),
        Commands::Doctor => doctor::run(&ctx),
    };
    if let Err(err) = &result {
        eprintln!("error: {err}");
    }
    std::process::exit(result_exit_code(&result));
}