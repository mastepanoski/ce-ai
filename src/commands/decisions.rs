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
        /// API key value to set (if omitted, prompts securely via stdin or displays current key status).
        #[arg(long, num_args = 0..=1)]
        key: Option<Option<String>>,
        /// Read API key value from standard input.
        #[arg(long)]
        stdin: bool,
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
    /// Evaluate execution risk policy for a command or tool invocation.
    CheckRisk {
        /// Tool name (e.g. "run_command", "write_to_file", "delete_file").
        tool: String,
        /// Command string or target path to evaluate.
        command: String,
        /// Optional prompt or context of the current task.
        #[arg(long)]
        task: Option<String>,
        /// Output in JSON format.
        #[arg(long)]
        json: bool,
        /// Display individual risk dimension scores.
        #[arg(long)]
        verbose: bool,
    },
    /// Evaluate work readiness and verification advisory for an ODD task or CE stage.
    CheckReadiness {
        /// Optional feature or branch name (defaults to active branch/feature).
        #[arg(long)]
        feature: Option<String>,
        /// Optional task brief or task description to evaluate.
        #[arg(long)]
        task: Option<String>,
        /// Optional Compound Engineering stage number (1..7).
        #[arg(long)]
        stage: Option<u32>,
        /// Output in JSON format.
        #[arg(long)]
        json: bool,
        /// Display individual dimension confidence scores.
        #[arg(long)]
        verbose: bool,
    },
    /// Evaluate model routing recommendation for a given task description.
    Route(crate::commands::models::RouteArgs),
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.action {
        Action::Status { json } => handle_status(ctx, *json),
        Action::Auth { key, stdin, check } => {
            handle_auth(ctx, key.as_ref().map(|o| o.as_deref()), *stdin, *check)
        }
        Action::Setup { preset } => handle_setup(ctx, preset),
        Action::Test { provider } => handle_test(ctx, provider.as_deref()),
        Action::CheckRisk {
            tool,
            command,
            task,
            json,
            verbose,
        } => handle_check_risk(ctx, tool, command, task.as_deref(), *json, *verbose),
        Action::CheckReadiness {
            feature,
            task,
            stage,
            json,
            verbose,
        } => handle_check_readiness(
            ctx,
            feature.as_deref(),
            task.as_deref(),
            *stage,
            *json,
            *verbose,
        ),
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
            },
            "risk": {
                "enabled": config.risk.enabled,
                "confirmation_threshold_pct": config.risk.thresholds.confirmation_threshold_pct,
                "deny_threshold_pct": config.risk.thresholds.deny_threshold_pct,
                "fallback": config.risk.fallback.as_str(),
            },
            "readiness": {
                "enabled": config.readiness.enabled,
                "ready_pct": config.readiness.thresholds.ready_pct,
                "warning_pct": config.readiness.thresholds.warning_pct,
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

    println!();
    println!("Risk Evaluation Engine");
    println!(
        "  Enabled:     {}",
        if config.risk.enabled { "yes" } else { "no" }
    );
    println!(
        "  Thresholds:  Confirmation >= {}%, Deny >= {}%",
        config.risk.thresholds.confirmation_threshold_pct,
        config.risk.thresholds.deny_threshold_pct
    );
    println!("  Fallback:    {}", config.risk.fallback.as_str());

    println!();
    println!("Readiness Advisory Engine");
    println!(
        "  Enabled:     {}",
        if config.readiness.enabled {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  Thresholds:  Ready >= {}%, Warning >= {}%",
        config.readiness.thresholds.ready_pct, config.readiness.thresholds.warning_pct
    );

    Ok(())
}

fn handle_auth(
    ctx: &Context,
    key: Option<Option<&str>>,
    from_stdin: bool,
    check: bool,
) -> Result<(), CeError> {
    let mut key_updated = false;

    if from_stdin {
        let read_key = crate::decisions::auth::read_api_key_from_stdin()?;
        let saved_path = save_api_key(&read_key, None)?;
        println!("Saved API key to OS Keyring and {}", saved_path.display());
        key_updated = true;
    } else if let Some(key_arg) = key {
        match key_arg {
            Some(k) => {
                eprintln!(
                    "warning: passing API key via command-line arguments exposes it in shell history and process lists; prefer interactive prompt via 'ce-ai decisions auth --key' or '--stdin'"
                );
                let saved_path = save_api_key(k, None)?;
                println!("Saved API key to OS Keyring and {}", saved_path.display());
                key_updated = true;
            }
            None => {
                let read_key = crate::decisions::auth::prompt_api_key_interactive(
                    "Enter TypeSafe/Jev API key: ",
                )?;
                match read_key {
                    Some(k) => {
                        let saved_path = save_api_key(&k, None)?;
                        println!("Saved API key to OS Keyring and {}", saved_path.display());
                        key_updated = true;
                    }
                    None => {
                        println!("No key entered; preserving current configuration.");
                    }
                }
            }
        }
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
    } else if !key_updated {
        match resolved.as_deref() {
            Some(k) => println!(
                "Current API key: configured ({})",
                crate::decisions::auth::mask_api_key(k)
            ),
            None => println!(
                "Current API key: not set (run 'ce-ai decisions auth --key' or export TYPESAFE_API_KEY)"
            ),
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
            risk: crate::decisions::RiskConfig {
                enabled: true,
                thresholds: crate::decisions::RiskThresholds::default(),
                fallback: crate::decisions::RiskFallbackPolicy::RequireConfirmation,
            },
            readiness: crate::decisions::ReadinessConfig {
                enabled: true,
                thresholds: crate::decisions::ReadinessThresholds::default(),
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
            risk: crate::decisions::RiskConfig {
                enabled: true,
                thresholds: crate::decisions::RiskThresholds::default(),
                fallback: crate::decisions::RiskFallbackPolicy::RequireConfirmation,
            },
            readiness: crate::decisions::ReadinessConfig {
                enabled: true,
                thresholds: crate::decisions::ReadinessThresholds::default(),
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
            risk: crate::decisions::RiskConfig {
                enabled: true,
                thresholds: crate::decisions::RiskThresholds::default(),
                fallback: crate::decisions::RiskFallbackPolicy::RequireConfirmation,
            },
            readiness: crate::decisions::ReadinessConfig {
                enabled: true,
                thresholds: crate::decisions::ReadinessThresholds::default(),
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
    println!("Next step: run 'ce-ai decisions auth' or export TYPESAFE_API_KEY.");

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

fn handle_check_risk(
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
    let config = state.decisions.unwrap_or_default();

    let provider_box: Option<Box<dyn DecisionProvider>> = if !config.enabled {
        None
    } else if config.provider == "mock" {
        Some(Box::new(MockDecisionProvider::new()))
    } else {
        Some(Box::new(JevProvider::new(config.jev.clone(), None)))
    };

    let engine = DecisionEngine::new(provider_box, config.mode);
    let evaluator = crate::decisions::RiskEvaluator::new(&config.risk, Some(&engine));
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
        crate::decisions::ExecutionPolicy::Allow => "ALLOW (Safe to execute)",
        crate::decisions::ExecutionPolicy::RequireConfirmation => {
            "REQUIRE_CONFIRMATION (User approval needed)"
        }
        crate::decisions::ExecutionPolicy::Deny => "DENY (Categorically blocked)",
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

fn handle_check_readiness(
    ctx: &Context,
    feature: Option<&str>,
    task: Option<&str>,
    stage: Option<u32>,
    as_json: bool,
    verbose: bool,
) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
        .unwrap_or_default();
    let config = state.decisions.clone().unwrap_or_default();

    let provider_box: Option<Box<dyn DecisionProvider>> = if !config.enabled {
        None
    } else if config.provider == "mock" {
        Some(Box::new(MockDecisionProvider::new()))
    } else {
        Some(Box::new(JevProvider::new(config.jev.clone(), None)))
    };

    let engine = DecisionEngine::new(provider_box, config.mode);
    let evaluator = crate::decisions::ReadinessEvaluator::new(&config.readiness, Some(&engine));

    let repo_root = ctx.repo_root();
    let diff_stat =
        crate::commands::workflow::git_probe(&repo_root, &["diff", "--stat"]).and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            }
        });

    let branch = crate::commands::workflow::probe_git_branch(&repo_root);
    let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());

    let feature_name = feature
        .map(String::from)
        .or_else(|| wf.as_ref().and_then(|w| w.feature_name.clone()))
        .or_else(|| branch.clone())
        .unwrap_or_else(|| "current-task".to_string());

    let result = if let Some(s) = stage {
        let stage_enum = crate::state::state::WorkflowStage::parse(&s.to_string())
            .unwrap_or(crate::state::state::WorkflowStage::WorkTdd);
        let s_name = stage_enum.as_str();
        let context = task.unwrap_or("Stage transition verification check");
        evaluator.evaluate_stage(&feature_name, s, s_name, context, diff_stat.as_deref())
    } else {
        // Check if odd/tasks/<feature>.md exists
        let mut brief_summary = task.unwrap_or("Organic task implementation").to_string();
        let mut checklist_status = String::new();

        let task_file = repo_root
            .join("odd")
            .join("tasks")
            .join(format!("{feature_name}.md"));
        if task_file.is_file() {
            if let Ok(content) = std::fs::read_to_string(&task_file) {
                brief_summary = content.clone();
                let checklist_lines: Vec<&str> = content
                    .lines()
                    .filter(|l| {
                        l.trim_start().starts_with("- [") || l.trim_start().starts_with("* [")
                    })
                    .collect();
                if !checklist_lines.is_empty() {
                    checklist_status = checklist_lines.join("\n");
                }
            }
        }

        if checklist_status.is_empty() {
            checklist_status = task.unwrap_or("- [x] Active task").to_string();
        }

        let is_ce_fsm = wf.as_ref().is_some_and(|w| {
            w.execution_mode == Some(crate::state::state::ExecutionMode::Compound)
        });

        if is_ce_fsm {
            let s_num = wf.as_ref().map(|w| w.stage.number()).unwrap_or(4);
            let s_name = wf.as_ref().map(|w| w.stage.as_str()).unwrap_or("work");
            evaluator.evaluate_stage(
                &feature_name,
                s_num,
                s_name,
                &brief_summary,
                diff_stat.as_deref(),
            )
        } else {
            evaluator.evaluate_odd(
                &feature_name,
                &brief_summary,
                &checklist_status,
                diff_stat.as_deref(),
            )
        }
    };

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&result).map_err(|e| CeError::Runtime(format!(
                "failed to serialize readiness result: {e}"
            )))?
        );
        return Ok(());
    }

    println!("Work Readiness Advisory: {}", result.status.indicator());
    println!("  Target:     {}", result.target);
    println!("  Workflow:   {}", result.workflow_mode);
    println!("  Score:      {:.1}%", result.composite_score * 100.0);
    if result.graduation_suggested {
        println!("  Graduation: RECOMMENDED (consider promoting to formal OpenSpec via 'ce-ai graduate')");
    }
    if result.fallback_applied {
        println!("  Fallback:   Active (deterministic workflow unaffected)");
    }
    println!("  Latency:    {}ms", result.latency_ms);

    if !result.advisory_notes.is_empty() {
        println!();
        println!("Advisory Guidance:");
        for note in &result.advisory_notes {
            println!("  - {note}");
        }
    }

    if verbose && !result.dimensions.is_empty() {
        println!();
        println!("Readiness Dimensions Breakdown:");
        for dim in &result.dimensions {
            let status = if dim.passed { "PASS" } else { "ATTN" };
            println!(
                "  - {:<26} {:>5.1}% [{status}]",
                dim.dimension,
                dim.confidence * 100.0
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_handle_check_risk_deterministic_denial() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let res = handle_check_risk(&ctx, "run_command", "rm -rf /", None, true, false);
        assert!(res.is_ok());

        let log_file = crate::decisions::risk_events_log_path(&ctx.config_dir);
        assert!(log_file.exists());
        let log_content = std::fs::read_to_string(&log_file).unwrap();
        assert!(log_content.contains("\"policy\":\"deny\""));
        assert!(log_content.contains("Deterministic denial"));
    }

    #[test]
    fn test_handle_check_risk_safe_read_only() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let res = handle_check_risk(&ctx, "run_command", "ls -la", None, true, false);
        assert!(res.is_ok());

        let log_file = crate::decisions::risk_events_log_path(&ctx.config_dir);
        assert!(log_file.exists());
        let log_content = std::fs::read_to_string(&log_file).unwrap();
        assert!(log_content.contains("\"policy\":\"allow\""));
        assert!(log_content.contains("Categorically safe"));
    }

    #[test]
    fn test_handle_setup_presets_configure_risk_and_readiness() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        handle_setup(&ctx, "recommended").unwrap();
        let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
        let decisions = state.decisions.unwrap();
        assert!(decisions.risk.enabled);
        assert_eq!(decisions.risk.thresholds.confirmation_threshold_pct, 60);
        assert_eq!(decisions.risk.thresholds.deny_threshold_pct, 90);
        assert_eq!(
            decisions.risk.fallback,
            crate::decisions::RiskFallbackPolicy::RequireConfirmation
        );

        assert!(decisions.readiness.enabled);
        assert_eq!(decisions.readiness.thresholds.ready_pct, 80);
        assert_eq!(decisions.readiness.thresholds.warning_pct, 60);
    }

    #[test]
    fn test_handle_check_readiness_fallback_unconfigured() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let res = handle_check_readiness(
            &ctx,
            Some("my-feat"),
            Some("Do something"),
            None,
            true,
            false,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_handle_check_readiness_stage_transition() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let res = handle_check_readiness(
            &ctx,
            Some("my-feat"),
            Some("Stage 4 check"),
            Some(4),
            false,
            true,
        );
        assert!(res.is_ok());
    }

    #[test]
    fn test_auth_clap_parsing_modes() {
        use clap::Parser;

        #[derive(Parser, Debug)]
        struct Cli {
            #[command(subcommand)]
            action: Action,
        }

        // 1. Plain auth (no flags)
        let parsed1 = Cli::try_parse_from(["cli", "auth"]).unwrap();
        match parsed1.action {
            Action::Auth { key, stdin, check } => {
                assert_eq!(key, None);
                assert!(!stdin);
                assert!(!check);
            }
            _ => panic!("expected Action::Auth"),
        }

        // 2. Auth with --stdin
        let parsed2 = Cli::try_parse_from(["cli", "auth", "--stdin"]).unwrap();
        match parsed2.action {
            Action::Auth { key, stdin, check } => {
                assert_eq!(key, None);
                assert!(stdin);
                assert!(!check);
            }
            _ => panic!("expected Action::Auth"),
        }

        // 3. Auth with --key without value
        let parsed3 = Cli::try_parse_from(["cli", "auth", "--key"]).unwrap();
        match parsed3.action {
            Action::Auth { key, stdin, check } => {
                assert_eq!(key, Some(None));
                assert!(!stdin);
                assert!(!check);
            }
            _ => panic!("expected Action::Auth"),
        }

        // 4. Auth with --key with value
        let parsed4 = Cli::try_parse_from(["cli", "auth", "--key", "ts-val-123"]).unwrap();
        match parsed4.action {
            Action::Auth { key, stdin, check } => {
                assert_eq!(key, Some(Some("ts-val-123".to_string())));
                assert!(!stdin);
                assert!(!check);
            }
            _ => panic!("expected Action::Auth"),
        }

        // 5. Auth with --stdin and --check
        let parsed5 = Cli::try_parse_from(["cli", "auth", "--stdin", "--check"]).unwrap();
        match parsed5.action {
            Action::Auth { key, stdin, check } => {
                assert_eq!(key, None);
                assert!(stdin);
                assert!(check);
            }
            _ => panic!("expected Action::Auth"),
        }
    }

    #[test]
    fn test_handle_auth_with_key_arg() {
        let temp = tempdir().unwrap();
        let creds_path = temp.path().join("credentials.toml");
        std::env::set_var("CE_AI_CREDENTIALS_PATH", creds_path.to_str().unwrap());

        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let res = handle_auth(&ctx, Some(Some("ts-unit-key-456")), false, false);
        assert!(res.is_ok());

        let resolved = resolve_api_key(None);
        assert_eq!(resolved.as_deref(), Some("ts-unit-key-456"));

        std::env::remove_var("CE_AI_CREDENTIALS_PATH");
    }

    #[test]
    fn test_handle_auth_non_interactive_no_flags() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().to_path_buf(),
            opencode_config_dir: temp.path().join("opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        // In non-interactive test runner, should simply display status without hanging
        let res = handle_auth(&ctx, None, false, false);
        assert!(res.is_ok());
    }
}
