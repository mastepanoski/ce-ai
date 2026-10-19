//! Operational guardrails and tool execution risk evaluation (`ce-ai guard`).
//!
//! Provides deterministic safety precedence and local-first semantic risk evaluation
//! for autonomous coding agents, preventing destructive commands, credential leaks,
//! and unauthorized git operations.

use clap::{Parser, Subcommand};

use crate::commands::Context;
use crate::decisions::{DecisionEngine, ExecutionPolicy, RiskEvaluator};
use crate::error::CeError;
use crate::state::state::{GuardLevel, GuardrailState, State};

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub command: GuardCommands,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum GuardCommands {
    /// Evaluate execution risk policy for a command or tool invocation
    Check {
        /// Target tool name (e.g. run_command, edit_file)
        tool: String,

        /// Command or payload string to inspect
        command: String,

        /// Optional task context description
        #[arg(long)]
        task: Option<String>,

        /// Emit machine-readable JSON output
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Print verbose risk dimension breakdown
        #[arg(long, short = 'v', default_value_t = false)]
        verbose: bool,
    },
    /// Report operational guardrails and pedagogical configuration status
    Status {
        /// Emit machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Quick-setup preset configuration for Guardrails (recommended, laya, kev, local, off)
    Setup {
        /// Setup preset: recommended (local-first / MLX), laya, kev, local, off
        #[arg(long, default_value = "recommended")]
        preset: String,

        /// Optional custom provider override
        #[arg(long)]
        provider: Option<String>,

        /// Optional endpoint override
        #[arg(long)]
        endpoint: Option<String>,
    },
    /// Run a self-test evaluation probe on sample commands
    Test,
    /// Persist pedagogical guardrail configuration for status reporting (Issue #114)
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
}

/// Executes the `ce-ai guard` subcommand dispatch.
pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.command {
        GuardCommands::Check {
            tool,
            command,
            task,
            json,
            verbose,
        } => run_guard_check(ctx, tool, command, task.as_deref(), *json, *verbose),
        GuardCommands::Status { json } => run_guard_status(ctx, *json),
        GuardCommands::Setup {
            preset,
            provider,
            endpoint,
        } => run_guard_setup(ctx, preset, provider.as_deref(), endpoint.as_deref()),
        GuardCommands::Test => run_guard_test(ctx),
        GuardCommands::Enable { level, harness } => {
            run_guard_enable(ctx, level, harness.as_deref())
        }
        GuardCommands::Disable { harness } => run_guard_disable(ctx, harness.as_deref()),
    }
}

/// Evaluates execution risk for a tool and command invocation.
pub fn run_guard_check(
    ctx: &Context,
    tool: &str,
    command: &str,
    task: Option<&str>,
    as_json: bool,
    verbose: bool,
) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
        .unwrap_or_default();
    let config = state.decisions.clone().unwrap_or_default();

    let root = ctx.repo_root();
    let branch = crate::commands::workflow::probe_git_branch(&root);
    let current_wf = state.current_workflow_for_branch(&root, branch.as_deref());
    let workflow_id = current_wf.as_ref().and_then(|w| w.feature_name.clone());

    let engine = DecisionEngine::from_config(&config);
    let evaluator = RiskEvaluator::new(&config.risk, Some(&engine))
        .with_config_dir(&ctx.config_dir)
        .with_workflow(workflow_id);
    let result = evaluator.evaluate(tool, command, task);

    // Persist to risk audit log
    let _ = crate::decisions::log_risk_event(&ctx.config_dir, &result);

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result)
                .map_err(|e| CeError::Runtime(format!("failed to serialize risk result: {e}")))?
        );
        return Ok(());
    }

    let policy_label = match result.policy {
        ExecutionPolicy::Allow => "ALLOW (Safe to execute)",
        ExecutionPolicy::RequireConfirmation => "REQUIRE_CONFIRMATION (User approval needed)",
        ExecutionPolicy::Deny => "DENY (Categorically blocked)",
    };

    println!("Risk Evaluation Policy: {policy_label}");
    println!("  Tool:       {}", result.tool);
    println!("  Command:    {}", result.command);
    if let Some(t) = &result.task {
        println!("  Task:       {t}");
    }
    println!("  Score:      {:.1}%", result.composite_risk_score * 100.0);
    println!("  Reason:     {}", result.reason);
    if result.fallback_applied {
        println!("  Fallback:   Active (provider unconfigured or unreachable)");
    }
    println!("  Latency:    {}ms", result.latency_ms);

    if verbose && !result.dimensions.is_empty() {
        println!();
        println!("Risk Dimensions Breakdown:");
        for dim in &result.dimensions {
            let status = if dim.elevated { "ELEVATED" } else { "safe" };
            println!(
                "  - {:<22} {:>5.1}% [{status}]",
                dim.dimension,
                dim.confidence * 100.0
            );
        }
    }

    Ok(())
}

/// Applies a quick-setup preset configuration for Guardrails.
pub fn run_guard_setup(
    ctx: &Context,
    preset: &str,
    provider_override: Option<&str>,
    endpoint_override: Option<&str>,
) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state =
        State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
            .unwrap_or_default();

    let clean = preset.trim().to_lowercase();
    let mut config = state.decisions.unwrap_or_default();

    match clean.as_str() {
        "recommended" => {
            config.enabled = true;
            config.mode = crate::decisions::DecisionMode::Active;
            config.risk.enabled = true;
            if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
                config.provider = "laya".into();
            } else {
                config.provider = "local".into();
            }
        }
        "laya" | "mlx" => {
            config.enabled = true;
            config.mode = crate::decisions::DecisionMode::Active;
            config.risk.enabled = true;
            config.provider = "laya".into();
            if let Some(ep) = endpoint_override {
                config.laya.endpoint = ep.to_string();
            }
        }
        "kev" => {
            config.enabled = true;
            config.mode = crate::decisions::DecisionMode::Active;
            config.risk.enabled = true;
            config.provider = "kev".into();
            if let Some(ep) = endpoint_override {
                config.kev.endpoint = ep.to_string();
            }
        }
        "local" | "mock" => {
            config.enabled = true;
            config.mode = crate::decisions::DecisionMode::Active;
            config.risk.enabled = true;
            config.provider = "mock".into();
        }
        "off" | "disabled" => {
            config.enabled = false;
            config.mode = crate::decisions::DecisionMode::Off;
            config.risk.enabled = false;
        }
        other => {
            return Err(CeError::Usage(format!(
                "unknown guard setup preset '{other}'. Valid presets: recommended, laya, kev, local, off"
            )));
        }
    }

    if let Some(prov) = provider_override {
        config.provider = prov.to_string();
    }

    state.decisions = Some(config.clone());
    state.save(&state_path)?;

    if !ctx.quiet {
        println!(
            "✓ Guardrails configuration updated (preset: {clean}, provider: {})",
            config.provider
        );
        println!(
            "  Status: {}",
            if config.risk.enabled {
                "Active"
            } else {
                "Disabled"
            }
        );
        println!(
            "  Thresholds: Confirm >= {}%, Deny >= {}%",
            config.risk.thresholds.confirmation_threshold_pct,
            config.risk.thresholds.deny_threshold_pct
        );
    }

    Ok(())
}

/// Runs hermetic test evaluation checks on sample commands.
pub fn run_guard_test(ctx: &Context) -> Result<(), CeError> {
    println!("🧪 Testing Agent Guardrail Pre-execution Checks...\n");

    let test_cases = [
        ("run_command", "git status", "Safe development command"),
        (
            "run_command",
            "git push --force origin main",
            "Destructive branch overwrite",
        ),
        (
            "run_command",
            "sudo rm -rf /",
            "Catastrophic filesystem destruction",
        ),
    ];

    for (tool, cmd, desc) in test_cases {
        let state_path = ctx.config_dir.join("state.json");
        let state =
            State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
                .unwrap_or_default();
        let config = state.decisions.clone().unwrap_or_default();
        let engine = DecisionEngine::from_config(&config);
        let evaluator =
            RiskEvaluator::new(&config.risk, Some(&engine)).with_config_dir(&ctx.config_dir);
        let result = evaluator.evaluate(tool, cmd, Some(desc));

        let icon = match result.policy {
            ExecutionPolicy::Allow => "✓ [ALLOW]",
            ExecutionPolicy::RequireConfirmation => "⚠️ [CONFIRM]",
            ExecutionPolicy::Deny => "🛑 [DENY]",
        };

        println!("{icon} Tool: {tool} | Command: '{cmd}'");
        println!(
            "   Policy: {:?} | Score: {:.1}% | Latency: {}ms",
            result.policy,
            result.composite_risk_score * 100.0,
            result.latency_ms
        );
        println!("   Reason: {}\n", result.reason);
    }

    Ok(())
}

/// Reports operational guardrails and pedagogical configuration status.
pub fn run_guard_status(ctx: &Context, json: bool) -> Result<(), CeError> {
    let state_path = ctx.state_path();
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
        .unwrap_or_default();

    let decisions_cfg = state.decisions.clone().unwrap_or_default();
    let guardrail_state = state.guardrail.clone();

    if json {
        let payload = serde_json::json!({
            "pedagogical": guardrail_state.unwrap_or_else(|| GuardrailState {
                enabled: false,
                level: GuardLevel::Junior,
                harness: None,
                updated_at: String::new(),
            }),
            "risk_engine": {
                "enabled": decisions_cfg.risk.enabled,
                "provider": decisions_cfg.provider,
                "mode": format!("{:?}", decisions_cfg.mode).to_lowercase(),
                "thresholds": decisions_cfg.risk.thresholds,
                "fallback": decisions_cfg.risk.fallback.as_str(),
            }
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("Agent Guardrails Status");
    println!(
        "  Operational Risk Engine: {}",
        if decisions_cfg.risk.enabled {
            "Active"
        } else {
            "Disabled"
        }
    );
    println!("  Provider:                {}", decisions_cfg.provider);
    println!(
        "  Mode:                    {}",
        format!("{:?}", decisions_cfg.mode).to_lowercase()
    );
    println!(
        "  Thresholds:              Confirm >= {}%, Deny >= {}%",
        decisions_cfg.risk.thresholds.confirmation_threshold_pct,
        decisions_cfg.risk.thresholds.deny_threshold_pct
    );
    println!(
        "  Fallback Policy:         {}",
        decisions_cfg.risk.fallback.as_str()
    );

    println!();
    match &guardrail_state {
        Some(g) if g.enabled => {
            println!("Pedagogical Guardrail Level: Enabled");
            println!("  Level:                   {}", g.level);
            println!(
                "  Scope:                   {}",
                g.harness.as_deref().unwrap_or("global")
            );
            println!("  Last Updated:            {}", g.updated_at);
        }
        Some(g) => {
            println!("Pedagogical Guardrail Level: Disabled");
            println!("  Last Updated:            {}", g.updated_at);
        }
        None => {
            println!("Pedagogical Guardrail Level: Disabled (not configured)");
        }
    }

    Ok(())
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

#[cfg(test)]
#[path = "tests/guard.rs"]
mod tests;
