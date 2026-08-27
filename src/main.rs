//! ce-ai: compound-engineering plugin manager CLI entry point (CC-1, CC-2).

#![forbid(unsafe_code)]

mod capture;
mod commands;
mod error;
mod harness;
mod opencode;
mod source;
mod state;
mod tui;

use std::path::PathBuf;

use clap::Parser;

use crate::commands::{registry::Commands, Context};
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

fn main() {
    let cli = Cli::parse();
    let ctx = match Context::resolve(cli.config_dir, cli.dry_run, cli.verbose, cli.quiet) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(err.exit_code());
        }
    };
    let result = crate::commands::registry::dispatch(&ctx, cli.command);
    if let Err(err) = &result {
        eprintln!("error: {err}");
    }
    std::process::exit(result_exit_code(&result));
}
