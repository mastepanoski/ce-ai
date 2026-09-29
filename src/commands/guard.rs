//! Pedagogical guardrail configuration (`ce-ai guard`) management (Issue #114).
//!
//! Persists an opt-in configuration marker for status and doctor reporting.

use clap::{Parser, Subcommand};

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::{GuardLevel, GuardrailState, State};

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub command: GuardCommands,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum GuardCommands {
    /// Persist pedagogical guardrail configuration for status reporting
    Enable {
        /// Stored configuration label: junior (default) or strict
        #[arg(long, default_value = "junior")]
        level: String,

        /// Target specific harness (defaults to global state)
        #[arg(long)]
        harness: Option<String>,
    },
    /// Disable persisted pedagogical guardrail configuration
    Disable {
        /// Require the configured harness scope to match before disabling
        #[arg(long)]
        harness: Option<String>,
    },
    /// Report persisted guardrail configuration and status
    Status {
        /// Emit machine-readable JSON
        #[arg(long)]
        json: bool,
    },
}

/// Executes the `ce-ai guard` subcommand dispatch.
pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.command {
        GuardCommands::Enable { level, harness } => {
            run_guard_enable(ctx, level, harness.as_deref())
        }
        GuardCommands::Disable { harness } => run_guard_disable(ctx, harness.as_deref()),
        GuardCommands::Status { json } => run_guard_status(ctx, *json),
    }
}

/// Enables the persisted pedagogical guardrail configuration marker.
pub fn run_guard_enable(
    ctx: &Context,
    level_str: &str,
    harness: Option<&str>,
) -> Result<(), CeError> {
    let level = GuardLevel::parse(level_str)?;
    let state_path = ctx.state_path();

    if ctx.dry_run {
        println!(
            "[dry-run] would enable pedagogical guardrail mode (level: {level}, harness: {})",
            harness.unwrap_or("global")
        );
        return Ok(());
    }

    let mut state = State::load(&state_path)?;
    state.guardrail = Some(GuardrailState {
        enabled: true,
        level,
        harness: harness.map(String::from),
        updated_at: chrono::Utc::now().to_rfc3339(),
    });

    state.save(&state_path)?;

    if !ctx.quiet {
        println!(
            "✓ Pedagogical guardrail mode enabled (level: {level}, scope: {})",
            harness.unwrap_or("global")
        );
    }

    Ok(())
}

/// Disables pedagogical guardrail mode cleanly.
pub fn run_guard_disable(ctx: &Context, harness: Option<&str>) -> Result<(), CeError> {
    let state_path = ctx.state_path();

    if ctx.dry_run {
        println!("[dry-run] would disable pedagogical guardrail mode");
        return Ok(());
    }

    let mut state = State::load(&state_path)?;
    if let Some(requested_harness) = harness {
        let configured_harness = state
            .guardrail
            .as_ref()
            .and_then(|guard| guard.harness.as_deref());
        if configured_harness != Some(requested_harness) {
            return Err(CeError::Usage(format!(
                "cannot disable guardrail for harness '{requested_harness}': configured scope is '{}'",
                configured_harness.unwrap_or("global")
            )));
        }
    }
    if let Some(guard) = &mut state.guardrail {
        guard.enabled = false;
        guard.updated_at = chrono::Utc::now().to_rfc3339();
    } else {
        state.guardrail = Some(GuardrailState {
            enabled: false,
            level: GuardLevel::Junior,
            harness: None,
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    state.save(&state_path)?;

    if !ctx.quiet {
        println!("✓ Pedagogical guardrail mode disabled");
    }

    Ok(())
}

/// Reports current guardrail status.
pub fn run_guard_status(ctx: &Context, json: bool) -> Result<(), CeError> {
    let state_path = ctx.state_path();
    let state = State::load(&state_path)?;

    if json {
        let payload = match &state.guardrail {
            Some(g) => serde_json::to_string_pretty(g)?,
            None => serde_json::to_string_pretty(&serde_json::json!({
                "enabled": false,
                "level": "junior",
                "harness": null,
                "updated_at": null
            }))?,
        };
        println!("{payload}");
        return Ok(());
    }

    match &state.guardrail {
        Some(g) if g.enabled => {
            println!("Pedagogical Guardrail Status: Enabled");
            println!("  Level: {}", g.level);
            println!("  Scope: {}", g.harness.as_deref().unwrap_or("global"));
            println!("  Last Updated: {}", g.updated_at);
        }
        Some(g) => {
            println!("Pedagogical Guardrail Status: Disabled");
            println!("  Last Updated: {}", g.updated_at);
        }
        None => {
            println!("Pedagogical Guardrail Status: Disabled (not configured)");
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/guard.rs"]
mod tests;
