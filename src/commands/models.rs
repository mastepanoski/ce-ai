//! `ce-ai models`: persist model assignments in state.json (MM-1), apply them
//! to opencode.json `agent.<slot>.model` (MM-2), and manage named profiles
//! with append-only snapshots (MM-3, MM-4). Model assignment is always
//! user-driven; ce-ai never seeds default models (#111).

use std::collections::BTreeMap;

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::agents::apply_agent_model;
use crate::state::profiles::{load_profile, save_profile, save_snapshot, Profile};
use crate::state::read_config;
use crate::state::state::State;

/// Parses harness CLI `models` output into sorted, deduped
/// `provider/model` identifiers (one per line; extra annotations after
/// whitespace are dropped).
fn parse_models_output(text: &str) -> Vec<String> {
    let mut models: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter_map(|line| line.split_whitespace().next())
        .filter(|token| {
            token.split_once('/').is_some_and(|(provider, model)| {
                !provider.is_empty() && !model.is_empty() && !model.contains('/')
            })
        })
        .map(str::to_string)
        .collect();
    models.sort();
    models.dedup();
    models
}

/// Reports whether a harness exposes a discoverable model catalog. Only
/// opencode ships a `models` CLI today; others fail explicitly instead of
/// showing another harness's list (#111).
pub fn discovery_supported(harness: &str) -> bool {
    harness.eq_ignore_ascii_case("opencode")
}

/// Discovers the models offered by the given harness installation.
/// opencode is queried via its CLI (`opencode models`) so pickers reflect
/// what the configured providers actually offer; unsupported harnesses
/// fail explicitly rather than fabricating data (#111).
pub fn discover_models(harness: &str) -> Result<Vec<String>, CeError> {
    if !discovery_supported(harness) {
        return Err(CeError::Usage(format!(
            "model discovery is not supported for harness '{harness}' yet; assign with `ce-ai models set --harness {harness} <slot> <provider/model>`"
        )));
    }
    let output = std::process::Command::new("opencode")
        .arg("models")
        .output()
        .map_err(|err| CeError::Runtime(format!("cannot execute 'opencode models': {err}")))?;
    if !output.status.success() {
        return Err(CeError::Runtime(format!(
            "'opencode models' exited with status {}",
            output.status.code().unwrap_or(-1)
        )));
    }
    let models = parse_models_output(&String::from_utf8_lossy(&output.stdout));
    if models.is_empty() {
        return Err(CeError::Runtime(
            "'opencode models' returned no usable entries".into(),
        ));
    }
    Ok(models)
}

/// Reads the effective `agent.<slot>.model` assignments for CE-known slots
/// from the given harness's live config (the display source of truth).
pub fn config_assignments(ctx: &Context, harness: &str) -> Vec<(String, String)> {
    let Ok(kind) = harness.parse::<crate::harness::HarnessKind>() else {
        return Vec::new();
    };
    let config_path = kind.config_path(&ctx.opencode_config_dir);
    let Ok(config) = read_config(&config_path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for slot in crate::harness::agents::CE_AGENT_SLOTS {
        if let Some(model) = config
            .get("agent")
            .and_then(|a| a.get(slot))
            .and_then(|e| e.get("model"))
            .and_then(|m| m.as_str())
            .filter(|m| !m.is_empty())
        {
            out.push((slot.to_string(), model.to_string()));
        }
    }
    out
}

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: ModelsCommand,
}

#[derive(clap::Subcommand)]
pub enum ModelsCommand {
    /// Assign a model (provider/model) to an agent slot.
    Set(SetArgs),
    /// List current model assignments.
    List,
    /// Named profile save/load with append-only snapshots.
    Profile(ProfileArgs),
    /// Query adaptive model route recommendation for a task description.
    Route(RouteArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct RouteArgs {
    /// Natural language task description or prompt to evaluate.
    pub task: String,
    /// Output in JSON format.
    #[arg(long)]
    pub json: bool,
    /// Verbose output with full decision answers and confidence metrics.
    #[arg(short, long)]
    pub verbose: bool,
    /// Check against existing static assignment for a specific slot.
    #[arg(long)]
    pub slot: Option<String>,
    /// Test an explicit model override.
    #[arg(long)]
    pub model: Option<String>,
    /// Default fallback model if unconfigured or unrouted.
    #[arg(long, default_value = "anthropic/claude-3-5-sonnet")]
    pub default_model: String,
}

#[derive(clap::Args)]
pub struct SetArgs {
    /// Agent slot, e.g. ce-brainstorm.
    pub slot: String,
    /// Model as provider/model, e.g. opencode-go/kimi-k2.6.
    pub model: String,
    /// Target harness config (opencode, claude, ...).
    #[arg(long, default_value = "opencode")]
    pub harness: String,
}

#[derive(clap::Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(clap::Subcommand)]
pub enum ProfileCommand {
    /// Save the current assignments as a named profile plus a snapshot.
    Save(ProfileNameArgs),
    /// Load a profile, restoring state.json and opencode.json.
    Load(ProfileNameArgs),
}

#[derive(clap::Args)]
pub struct ProfileNameArgs {
    pub name: String,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.command {
        ModelsCommand::Set(args) => set(ctx, &args.harness, &args.slot, &args.model),
        ModelsCommand::List => list(ctx),
        ModelsCommand::Profile(profile) => match &profile.command {
            ProfileCommand::Save(args) => save(ctx, &args.name),
            ProfileCommand::Load(args) => load(ctx, &args.name),
        },
        ModelsCommand::Route(args) => route(ctx, args),
    }
}

pub fn route(ctx: &Context, args: &RouteArgs) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())
        .unwrap_or_default();
    let decisions_config = state.decisions.unwrap_or_default();
    let routing_config = &decisions_config.routing;

    let slot_assignment = args.slot.as_deref().and_then(|slot| {
        state
            .model_assignments
            .get(slot)
            .map(|a| format!("{}/{}", a.provider_id, a.model_id))
    });

    let engine = crate::decisions::DecisionEngine::from_config(&decisions_config);
    let router = crate::decisions::routing::ModelRouter::new(routing_config, Some(&engine));
    let res = router.route(
        &args.task,
        &args.default_model,
        args.model.as_deref(),
        slot_assignment.as_deref(),
    );

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&res).map_err(|e| CeError::State(e.to_string()))?
        );
    } else {
        println!("Task: {}", res.task);
        println!("Recommended Class: {}", res.recommended_class);
        println!("Resolved Model: {}", res.resolved_model);
        if res.fallback_applied {
            println!("Fallback Applied: yes ({})", res.rationale);
        } else {
            println!("Rationale: {}", res.rationale);
        }
        if args.verbose {
            println!("Dimensions:");
            println!("  Complexity: {}", res.complexity);
            println!("  Needs Reasoning: {}", res.needs_reasoning);
            println!("  Needs Large Context: {}", res.needs_large_context);
            println!("  Risk: {}", res.risk);
            if res.latency_ms > 0 {
                println!("Latency: {}ms", res.latency_ms);
            }
        }
    }

    Ok(())
}

/// Renders assignments as `slot -> "provider/model"` for profiles and snapshots.
fn assignments_map(state: &State) -> BTreeMap<String, String> {
    state
        .model_assignments
        .iter()
        .map(|(slot, assignment)| {
            (
                slot.clone(),
                format!("{}/{}", assignment.provider_id, assignment.model_id),
            )
        })
        .collect()
}

pub(crate) fn set(ctx: &Context, harness: &str, slot: &str, model: &str) -> Result<(), CeError> {
    let (provider_id, model_id) = model
        .split_once('/')
        .filter(|(provider, model)| !provider.is_empty() && !model.is_empty())
        .ok_or_else(|| CeError::Usage(format!("model must be provider/model, got {model:?}")))?;

    let kind = harness.parse::<crate::harness::HarnessKind>()?;
    if !crate::harness::agents::supports_agent_definitions(&kind) {
        return Err(CeError::Usage(format!(
            "harness '{harness}' has no agent-map config; cannot assign models"
        )));
    }
    let home_dir = crate::harness::home_dir_from_ctx(ctx);
    let config_dir = if kind == crate::harness::HarnessKind::Opencode {
        ctx.opencode_config_dir.clone()
    } else {
        kind.harness_dir(&home_dir)
    };
    let config_path = kind.config_path(&config_dir);

    let state_path = ctx.config_dir.join("state.json");
    let mut state =
        State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())?;
    let before = assignments_map(&state);

    // Merge into the target harness config first so a config failure leaves
    // state untouched.
    let config = read_config(&config_path)?;
    if !config
        .get("agent")
        .and_then(|agents| agents.get(slot))
        .is_some_and(serde_json::Value::is_object)
    {
        eprintln!("warning: unknown agent slot {slot:?}; assignment persisted");
    }
    crate::harness::agents::apply_agent_model(&config_path, slot, model)?;

    state.set_model_assignment(slot, provider_id, model_id);
    state.save(&state_path)?;
    let after = assignments_map(&state);

    // Append-only snapshot around the change (MM-4).
    save_snapshot(&ctx.config_dir.join("profiles"), "state", &before, &after)?;
    if !ctx.quiet {
        println!("models: set {harness}/{slot} = {model}");
    }
    Ok(())
}

/// Compares `state.json` assignments against the `opencode.json` agent map
/// and returns human-readable drift findings (doctor; issue #111):
/// - state slot missing from config
/// - state and config disagreeing on the model
/// - CE-known slots present in config but untracked in state
///
/// Unknown third-party agent slots without state entries are ignored.
pub fn model_drift_findings(state: &State, config: &serde_json::Value) -> Vec<String> {
    let mut findings = Vec::new();
    let agents = config.get("agent");
    let config_model = |slot: &str| -> Option<String> {
        agents
            .and_then(|a| a.get(slot))
            .and_then(|entry| entry.get("model"))
            .and_then(|m| m.as_str())
            .filter(|m| !m.is_empty())
            .map(str::to_string)
    };
    for (slot, assignment) in &state.model_assignments {
        match config_model(slot) {
            None => findings.push(format!(
                "model-assignment-drift: slot '{slot}' missing from opencode.json agent map"
            )),
            Some(model) => {
                let effective = format!("{}/{}", assignment.provider_id, assignment.model_id);
                if model != effective {
                    findings.push(format!(
                        "model-assignment-drift: slot '{slot}' config='{model}' state='{effective}' (run 'ce-ai sync' to reconcile)"
                    ));
                }
            }
        }
    }
    if let Some(serde_json::Value::Object(map)) = agents {
        for slot in map.keys() {
            let is_ce_slot = crate::harness::agents::CE_AGENT_SLOTS.contains(&slot.as_str());
            if is_ce_slot
                && !state.model_assignments.contains_key(slot)
                && config_model(slot).is_some()
            {
                findings.push(format!(
                    "model-assignment-drift: slot '{slot}' present in opencode.json but untracked in state.json (run 'ce-ai sync' to reconcile)"
                ));
            }
        }
    }
    findings
}

/// Evaluates whether an informational note should be emitted regarding unconfigured
/// mid-tier persona dispatch for `ce-code-review`.
pub fn check_code_review_mid_tier_note(
    state: &State,
    config: &serde_json::Value,
) -> Option<String> {
    let has_model = |slot: &str| -> bool {
        let in_config = config
            .get("agent")
            .and_then(|a| a.get(slot))
            .and_then(|e| e.get("model"))
            .and_then(|m| m.as_str())
            .is_some_and(|m| !m.is_empty());
        let in_state = state.model_assignments.contains_key(slot);
        in_config || in_state
    };

    let mid_tier = crate::harness::agents::CODE_REVIEW_MID_TIER_SLOT;
    if has_model("ce-code-review") && !has_model(mid_tier) {
        Some(format!(
            "ce-code-review has a model assigned but '{mid_tier}' is not configured; persona-level cost tiering has no explicit target on opencode and may silently fall back to the session model (run 'ce-ai models set --harness opencode {mid_tier} <provider/model>' to configure)"
        ))
    } else {
        None
    }
}

/// Imports effective `opencode.json` model assignments into `state`,
/// returning the imported `(slot, model)` pairs. Config is treated as the
/// live truth because users may edit it outside ce-ai (#111); opencode.json
/// itself is never modified here.
pub fn import_config_assignments(
    state: &mut State,
    config: &serde_json::Value,
) -> Vec<(String, String)> {
    let mut imported = Vec::new();
    let Some(serde_json::Value::Object(agents)) = config.get("agent") else {
        return imported;
    };
    for (slot, entry) in agents {
        let Some(model) = entry.get("model").and_then(|m| m.as_str()) else {
            continue;
        };
        if model.is_empty() || !model.contains('/') {
            continue;
        }
        let matches_state = state
            .model_assignments
            .get(slot)
            .is_some_and(|a| format!("{}/{}", a.provider_id, a.model_id) == model);
        if !matches_state {
            if let Some((provider_id, model_id)) = model.split_once('/') {
                state.set_model_assignment(slot, provider_id, model_id);
                imported.push((slot.clone(), model.to_string()));
            }
        }
    }
    imported
}

/// Removes state assignments whose slot no longer exists in the harness
/// config agent map — config wins in both directions (additions AND
/// deletions). Returns the purged slots. No-op when the config has no
/// `agent` map at all (conservative: an absent section is not treated as
/// a bulk deletion).
pub fn purge_stale_assignments(state: &mut State, config: &serde_json::Value) -> Vec<String> {
    let Some(serde_json::Value::Object(agents)) = config.get("agent") else {
        return Vec::new();
    };
    let stale: Vec<String> = state
        .model_assignments
        .keys()
        .filter(|slot| !agents.contains_key(*slot))
        .cloned()
        .collect();
    for slot in &stale {
        state.model_assignments.remove(slot);
    }
    stale
}

/// Formats model assignments for display, presenting tiering sub-slots
/// distinguished from top-level workflow stage slots.
pub fn format_model_assignments(
    assignments: &std::collections::BTreeMap<String, crate::state::state::ModelAssignment>,
) -> Vec<String> {
    if assignments.is_empty() {
        return vec!["models: none".to_string()];
    }

    let mut lines = Vec::new();
    let mid_tier_slot = crate::harness::agents::CODE_REVIEW_MID_TIER_SLOT;
    let mid_tier_assignment = assignments.get(mid_tier_slot);

    for (slot, assignment) in assignments {
        if slot == mid_tier_slot {
            // Rendered either under its parent or standalone below
            continue;
        }
        lines.push(format!(
            "{slot}: {}/{}",
            assignment.provider_id, assignment.model_id
        ));
        if slot == "ce-code-review" {
            if let Some(sub) = mid_tier_assignment {
                lines.push(format!(
                    "  └─ mid-tier ({mid_tier_slot}): {}/{}",
                    sub.provider_id, sub.model_id
                ));
            }
        }
    }

    // Standalone fallback if ce-code-review was not assigned but mid-tier was
    if let Some(sub) = mid_tier_assignment {
        if !assignments.contains_key("ce-code-review") {
            lines.push(format!(
                "{mid_tier_slot} (mid-tier sub-slot): {}/{}",
                sub.provider_id, sub.model_id
            ));
        }
    }

    lines
}

fn list(ctx: &Context) -> Result<(), CeError> {
    let state = State::load_with_workspace_overrides(
        &ctx.config_dir.join("state.json"),
        ctx.workspace_root.as_deref(),
    )?;
    for line in format_model_assignments(&state.model_assignments) {
        println!("{line}");
    }
    Ok(())
}

fn save(ctx: &Context, name: &str) -> Result<(), CeError> {
    let state = State::load_with_workspace_overrides(
        &ctx.config_dir.join("state.json"),
        ctx.workspace_root.as_deref(),
    )?;
    let models = assignments_map(&state);
    let root = ctx.config_dir.join("profiles");
    save_profile(
        &root,
        &Profile {
            name: name.into(),
            created_at: Utc::now().to_rfc3339(),
            models: models.clone(),
        },
    )?;
    save_snapshot(&root, name, &models, &models)?;
    if !ctx.quiet {
        println!("models: profile {name} saved");
    }
    Ok(())
}

fn load(ctx: &Context, name: &str) -> Result<(), CeError> {
    let profile = load_profile(&ctx.config_dir.join("profiles"), name)?;
    let state_path = ctx.config_dir.join("state.json");
    let mut state =
        State::load_with_workspace_overrides(&state_path, ctx.workspace_root.as_deref())?;
    state.model_assignments.clear();
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    for (slot, model) in &profile.models {
        let (provider_id, model_id) = model.split_once('/').ok_or_else(|| {
            CeError::Runtime(format!("profile {name} holds a malformed model {model:?}"))
        })?;
        state.set_model_assignment(slot, provider_id, model_id);
        apply_agent_model(&opencode_json, slot, model)?;
    }
    state.save(&state_path)?;
    if !ctx.quiet {
        println!("models: profile {name} loaded");
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/models.rs"]
mod tests;
