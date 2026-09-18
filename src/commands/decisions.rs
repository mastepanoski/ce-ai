//! CLI subcommand for managing and diagnosing the Decision Engine (`ce-ai decisions`).

use clap::{Args as ClapArgs, Subcommand};
use serde_json::json;

use crate::commands::Context;
use crate::decisions::auth::{resolve_api_key, save_api_key};
use crate::decisions::budget::{BudgetConfig, BudgetTracker};
use crate::decisions::jev::{JevConfig, JevProvider};
use crate::decisions::mock::MockDecisionProvider;
use crate::decisions::types::{
    DecisionAnswer, DecisionContext, DecisionMode, DecisionQuestion, DecisionRequest,
};
use crate::decisions::{DecisionEngine, DecisionProvider};
use crate::error::CeError;
use crate::state::state::{DecisionsConfig, State};

#[derive(ClapArgs, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    /// Display Decision Engine status, provider health, API key status, and budget consumption.
    Status {
        /// Output in JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Configure or test Decision Provider authentication credentials.
    Auth {
        /// API key value to set (if omitted, displays current key status).
        #[arg(long)]
        key: Option<String>,
        /// Verify provider connectivity with the resolved API key.
        #[arg(long)]
        check: bool,
    },
    /// Quick-setup wizard or preset configuration for the Decision Engine.
    Setup {
        /// Preset configuration: recommended (active Jev + $5 budget), shadow (evaluates without enforcement), or local (mock/offline).
        #[arg(long, default_value = "recommended")]
        preset: String,
    },
    /// Test decision evaluation with a sample structured query.
    Test {
        /// Override provider to test ("jev" or "mock").
        #[arg(long)]
        provider: Option<String>,
    },
    /// Evaluate model routing recommendation for a given task description.
    Route(crate::commands::models::RouteArgs),
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.action {
        Action::Status { json } => handle_status(ctx, *json),
        Action::Auth { key, check } => handle_auth(ctx, key.as_deref(), *check),
        Action::Setup { preset } => handle_setup(ctx, preset),
        Action::Test { provider } => handle_test(ctx, provider.as_deref()),
        Action::Route(route_args) => crate::commands::models::route(ctx, route_args),
    }
}

fn handle_status(ctx: &Context, as_json: bool) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())?;
    let config = state.decisions.unwrap_or_default();

    if !config.enabled {
        if as_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "enabled": false,
                    "message": "Decision Engine is disabled in configuration."
                }))
                .unwrap_or_default()
            );
        } else {
            println!("Decision Engine is disabled.");
            println!(
                "To enable and configure presets, run 'ce-ai decisions setup --preset recommended'."
            );
        }
        return Ok(());
    }

    let resolved_key = resolve_api_key(None);
    let mut tracker = BudgetTracker::new(config.budget.clone(), None);
    let _ = tracker.can_execute(); // trigger rollover check

    let (provider_box, health): (Option<Box<dyn DecisionProvider>>, _) =
        if config.provider == "mock" {
            let mock = MockDecisionProvider::new();
            let health = mock.check_health()?;
            (Some(Box::new(mock)), health)
        } else {
            let jev = JevProvider::new(config.jev.clone(), None);
            let health = jev.check_health()?;
            (Some(Box::new(jev)), health)
        };

    let _engine = DecisionEngine::new(provider_box, config.mode);

    if as_json {
        let payload = json!({
            "enabled": config.enabled,
            "provider": config.provider,
            "mode": config.mode.as_str(),
            "model": config.jev.model,
            "has_api_key": resolved_key.is_some(),
            "health": {
                "available": health.available,
                "latency_ms": health.latency_ms,
                "message": health.message,
            },
            "budget": {
                "monthly_spend_usd": tracker.accumulated_spend_usd(),
                "monthly_limit_usd": config.budget.max_monthly_usd(),
                "total_monthly_requests": tracker.total_monthly_requests(),
                "session_requests": tracker.session_requests(),
                "max_session_requests": config.budget.max_session_requests,
                "consecutive_failures": tracker.consecutive_failures(),
            }
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return Ok(());
    }

    println!("Decision Engine Status");
    println!(
        "  Enabled:     {}",
        if config.enabled { "yes" } else { "no" }
    );
    println!("  Provider:    {}", config.provider);
    println!("  Mode:        {}", config.mode);
    println!("  Model:       {}", config.jev.model);

    match resolved_key.as_deref() {
        Some(_) => println!("  API Key:     configured"),
        None => println!("  API Key:     not set (TYPESAFE_API_KEY missing)"),
    }

    if health.available {
        println!("  Health:      OK ({}ms latency)", health.latency_ms);
    } else {
        println!("  Health:      unavailable ({})", health.message);
    }

    println!();
    println!("Budget & Consumption");
    println!(
        "  Monthly Spend: ${:.2} / ${:.2} ({:.1}%)",
        tracker.accumulated_spend_usd(),
        config.budget.max_monthly_usd(),
        if config.budget.max_monthly_usd() > 0.0 {
            (tracker.accumulated_spend_usd() / config.budget.max_monthly_usd()) * 100.0
        } else {
            0.0
        }
    );
    println!(
        "  Session Req:   {} / {}",
        tracker.session_requests(),
        config.budget.max_session_requests
    );
    println!(
        "  Failures:      {} consecutive (circuit closed)",
        tracker.consecutive_failures()
    );

    Ok(())
}

fn handle_auth(ctx: &Context, key: Option<&str>, check: bool) -> Result<(), CeError> {
    if let Some(k) = key {
        let saved_path = save_api_key(k, None)?;
        println!("Saved API key to {}", saved_path.display());
    }

    let resolved = resolve_api_key(None);

    if check {
        let state_path = ctx.config_dir.join("state.json");
        let state =
            State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
                .unwrap_or_default();
        let config = state.decisions.unwrap_or_default();

        let jev = JevProvider::new(config.jev, None);
        let health = jev.check_health()?;

        if health.available {
            println!(
                "Authentication verification: OK ({}ms latency)",
                health.latency_ms
            );
        } else {
            eprintln!("Authentication verification failed: {}", health.message);
        }
    } else if key.is_none() {
        match resolved.as_deref() {
            Some(_) => println!("Current API key: configured"),
            None => println!("Current API key: not set (run 'ce-ai decisions auth --key <KEY>' or export TYPESAFE_API_KEY)"),
        }
    }

    Ok(())
}

fn handle_setup(ctx: &Context, preset_name: &str) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state =
        State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
            .unwrap_or_default();

    let clean = preset_name.trim().to_lowercase();
    let config = match clean.as_str() {
        "recommended" => DecisionsConfig {
            enabled: true,
            provider: "jev".into(),
            mode: DecisionMode::Active,
            budget: BudgetConfig {
                max_monthly_cents: 500,
                max_session_requests: 100,
                timeout_ms: 1000,
                max_consecutive_failures: 3,
                cooloff_secs: 60,
            },
            jev: JevConfig::default(),
            routing: crate::decisions::ModelRoutingConfig {
                enabled: true,
                models: crate::decisions::ModelClassCatalog {
                    fast: Some("anthropic/claude-3-5-haiku".into()),
                    standard: Some("anthropic/claude-3-5-sonnet".into()),
                    reasoning: Some("anthropic/claude-3-7-sonnet".into()),
                },
                thresholds: crate::decisions::RoutingThresholds::default(),
            },
            skills: crate::decisions::SkillRoutingConfig {
                enabled: true,
                minimum_confidence_pct: 70,
            },
        },
        "shadow" => DecisionsConfig {
            enabled: true,
            provider: "jev".into(),
            mode: DecisionMode::Shadow,
            budget: BudgetConfig {
                max_monthly_cents: 500,
                max_session_requests: 100,
                timeout_ms: 1000,
                max_consecutive_failures: 3,
                cooloff_secs: 60,
            },
            jev: JevConfig::default(),
            routing: crate::decisions::ModelRoutingConfig {
                enabled: true,
                models: crate::decisions::ModelClassCatalog {
                    fast: Some("anthropic/claude-3-5-haiku".into()),
                    standard: Some("anthropic/claude-3-5-sonnet".into()),
                    reasoning: Some("anthropic/claude-3-7-sonnet".into()),
                },
                thresholds: crate::decisions::RoutingThresholds::default(),
            },
            skills: crate::decisions::SkillRoutingConfig {
                enabled: true,
                minimum_confidence_pct: 70,
            },
        },
        "local" => DecisionsConfig {
            enabled: true,
            provider: "mock".into(),
            mode: DecisionMode::Active,
            budget: BudgetConfig::default(),
            jev: JevConfig::default(),
            routing: crate::decisions::ModelRoutingConfig {
                enabled: true,
                models: crate::decisions::ModelClassCatalog {
                    fast: Some("mock/fast".into()),
                    standard: Some("mock/standard".into()),
                    reasoning: Some("mock/reasoning".into()),
                },
                thresholds: crate::decisions::RoutingThresholds::default(),
            },
            skills: crate::decisions::SkillRoutingConfig {
                enabled: true,
                minimum_confidence_pct: 70,
            },
        },
        _ => {
            return Err(CeError::Usage(format!(
                "invalid preset '{preset_name}'. Valid presets: recommended, shadow, local"
            )));
        }
    };

    state.decisions = Some(config.clone());
    let serialized = serde_json::to_string_pretty(&state)
        .map_err(|e| CeError::State(format!("failed to serialize updated state.json: {e}")))?;

    crate::state::write_atomic(&state_path, serialized.as_bytes())?;

    println!("Decision Engine configured with preset '{}':", clean);
    println!("  Provider: {}", config.provider);
    println!("  Mode:     {}", config.mode);
    println!(
        "  Budget:   ${:.2} monthly ceiling",
        config.budget.max_monthly_usd()
    );
    println!();
    println!("Next step: run 'ce-ai decisions auth --key <KEY>' or export TYPESAFE_API_KEY.");

    Ok(())
}

fn handle_test(ctx: &Context, provider_override: Option<&str>) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
        .unwrap_or_default();
    let config = state.decisions.unwrap_or_default();

    let target_provider = provider_override.unwrap_or(&config.provider);

    let provider: Box<dyn DecisionProvider> = if target_provider == "mock" {
        Box::new(
            MockDecisionProvider::new()
                .with_canned_answer(
                    "is_safe",
                    DecisionAnswer::Boolean {
                        value: true,
                        confidence: 0.96,
                    },
                )
                .with_canned_answer(
                    "category",
                    DecisionAnswer::Choice {
                        selected: "refactor".into(),
                        confidence: 0.91,
                        probabilities: std::collections::BTreeMap::new(),
                    },
                ),
        )
    } else {
        Box::new(JevProvider::new(config.jev, None))
    };

    let req = DecisionRequest::new(DecisionContext::new("Sample test task from ce-ai CLI"))
        .with_question(DecisionQuestion::boolean("is_safe", "Is this change safe?"))
        .with_question(DecisionQuestion::choice(
            "category",
            "Task category",
            vec!["feature", "bugfix", "refactor"],
        ));

    println!(
        "Evaluating sample decision with provider '{}'...",
        provider.name()
    );
    let start = std::time::Instant::now();
    let resp = provider.evaluate(req)?;
    let total_latency = start.elapsed().as_millis() as u64;

    println!(
        "Decision Response (latency: {}ms, provider latency: {}ms):",
        total_latency, resp.latency_ms
    );
    for (qid, ans) in &resp.answers {
        match ans {
            DecisionAnswer::Boolean { value, confidence } => {
                println!(
                    "  - {qid}: {value} (confidence: {:.1}%)",
                    confidence * 100.0
                );
            }
            DecisionAnswer::Choice {
                selected,
                confidence,
                ..
            } => {
                println!(
                    "  - {qid}: {selected} (confidence: {:.1}%)",
                    confidence * 100.0
                );
            }
            DecisionAnswer::Score { score, confidence } => {
                println!(
                    "  - {qid}: {score} (confidence: {:.1}%)",
                    confidence * 100.0
                );
            }
        }
    }

    Ok(())
}
