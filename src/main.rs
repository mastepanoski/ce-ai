//! ce-ai: Workflow orchestration and governance for Compound Engineering across AI coding agents.

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

use std::path::PathBuf;

use clap::Parser;

use ce_ai::commands::{registry::Commands, Context};
use ce_ai::error::result_exit_code;

#[derive(Parser)]
#[command(
    name = "ce-ai",
    about = "Workflow orchestration and governance for Compound Engineering across AI coding agents",
    version
)]
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
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            ce_ai::source::binary_release::cleanup_stale_update_files(dir);
        }
    }

    let cli = Cli::parse();
    let ctx = match Context::resolve(cli.config_dir, cli.dry_run, cli.verbose, cli.quiet) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(err.exit_code());
        }
    };
    let is_self_update = matches!(cli.command, Some(Commands::SelfUpdate(_)));

    let state = ce_ai::state::state::State::load_with_workspace_overrides(
        &ctx.config_dir.join("state.json"),
        ctx.workspace_root.as_deref(),
    )
    .unwrap_or_default();

    if !is_self_update {
        ce_ai::source::update_notifier::maybe_trigger_background_check(&ctx, &state);
    }

    let result = ce_ai::commands::registry::dispatch(&ctx, cli.command);
    if let Err(err) = &result {
        eprintln!("error: {err}");
    }

    if !is_self_update {
        ce_ai::source::update_notifier::maybe_print_update_notification(&ctx, &state);
    }

    std::process::exit(result_exit_code(&result));
}
