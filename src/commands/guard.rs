//! Pedagogical Guardrail Mode (`ce-ai guard`) management (Issue #114).
//!
//! Provides opt-in oversight for junior developers preventing vibe coding,
//! supporting ISO/IEC 42001 and NIST AI RMF 1.0 human-in-the-loop requirements.

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
    /// Enable pedagogical guardrail mode for junior developer oversight
    Enable {
        /// Oversight intensity: junior (default, batched) or strict (per-module)
        #[arg(long, default_value = "junior")]
        level: String,

        /// Target specific harness (defaults to global state)
        #[arg(long)]
        harness: Option<String>,
    },
    /// Disable pedagogical guardrail mode cleanly
    Disable {
        /// Target specific harness (defaults to global state)
        #[arg(long)]
        harness: Option<String>,
    },
    /// Report current guardrail status and integrity
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

/// Enables pedagogical guardrail mode.
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
pub fn run_guard_disable(ctx: &Context, _harness: Option<&str>) -> Result<(), CeError> {
    let state_path = ctx.state_path();

    if ctx.dry_run {
        println!("[dry-run] would disable pedagogical guardrail mode");
        return Ok(());
    }

    let mut state = State::load(&state_path)?;
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
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_ctx() -> (TempDir, Context) {
        let tmp = TempDir::new().unwrap();
        let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
        (tmp, ctx)
    }

    #[test]
    fn guard_enable_persists_state_and_level() {
        let (_tmp, ctx) = test_ctx();
        run_guard_enable(&ctx, "strict", Some("claude")).unwrap();

        let state = State::load(&ctx.state_path()).unwrap();
        let guard = state.guardrail.expect("guardrail should exist");
        assert!(guard.enabled);
        assert_eq!(guard.level, GuardLevel::Strict);
        assert_eq!(guard.harness.as_deref(), Some("claude"));
    }

    #[test]
    fn guard_disable_cleans_flag() {
        let (_tmp, ctx) = test_ctx();
        run_guard_enable(&ctx, "junior", None).unwrap();
        run_guard_disable(&ctx, None).unwrap();

        let state = State::load(&ctx.state_path()).unwrap();
        let guard = state.guardrail.expect("guardrail should exist");
        assert!(!guard.enabled);
    }

    #[test]
    fn guard_invalid_level_fails_fast() {
        let (_tmp, ctx) = test_ctx();
        let err = run_guard_enable(&ctx, "extreme", None).unwrap_err();
        assert!(matches!(err, CeError::Usage(_)));
    }

    #[test]
    fn guard_dry_run_writes_nothing() {
        let (_tmp, mut ctx) = test_ctx();
        ctx.dry_run = true;
        run_guard_enable(&ctx, "strict", None).unwrap();

        assert!(!ctx.state_path().exists());
    }
}
