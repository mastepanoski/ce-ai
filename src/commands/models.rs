//! `ce-ai models`: persist model assignments in state.json (MM-1), apply them
//! to opencode.json `agent.<slot>.model`/`variant` (MM-2), and manage named
//! profiles with append-only snapshots (MM-3, MM-4).

use std::collections::BTreeMap;

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::{apply_model_assignment, read_config};
use crate::state::profiles::{load_profile, save_profile, save_snapshot, Profile};
use crate::state::state::State;

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
}

#[derive(clap::Args)]
pub struct SetArgs {
    /// Agent slot, e.g. sdd-explore.
    pub slot: String,
    /// Model as provider/model, e.g. opencode-go/kimi-k2.6.
    pub model: String,
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
        ModelsCommand::Set(args) => set(ctx, &args.slot, &args.model),
        ModelsCommand::List => list(ctx),
        ModelsCommand::Profile(profile) => match &profile.command {
            ProfileCommand::Save(args) => save(ctx, &args.name),
            ProfileCommand::Load(args) => load(ctx, &args.name),
        },
    }
}

/// Renders assignments as `slot -> "provider/model"` for profiles and snapshots.
fn assignments_map(state: &State) -> BTreeMap<String, String> {
    state
        .model_assignments
        .iter()
        .map(|(slot, assignment)| (slot.clone(), format!("{}/{}", assignment.provider_id, assignment.model_id)))
        .collect()
}

fn set(ctx: &Context, slot: &str, model: &str) -> Result<(), CeError> {
    let (provider_id, model_id) = model
        .split_once('/')
        .filter(|(provider, model)| !provider.is_empty() && !model.is_empty())
        .ok_or_else(|| CeError::Usage(format!("model must be provider/model, got {model:?}")))?;

    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    let before = assignments_map(&state);

    // Merge into opencode.json first so a config failure leaves state untouched.
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    let config = read_config(&opencode_json)?;
    if !config.get("agent").and_then(|agents| agents.get(slot)).is_some_and(serde_json::Value::is_object) {
        eprintln!("warning: unknown agent slot {slot:?}; assignment persisted");
    }
    apply_model_assignment(&opencode_json, slot, model)?;

    state.set_model_assignment(slot, provider_id, model_id);
    state.save(&state_path)?;
    let after = assignments_map(&state);

    // Append-only snapshot around the change (MM-4).
    save_snapshot(&ctx.config_dir.join("profiles"), "state", &before, &after)?;
    if !ctx.quiet {
        println!("models: set {slot} = {model}");
    }
    Ok(())
}

fn list(ctx: &Context) -> Result<(), CeError> {
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    if state.model_assignments.is_empty() {
        println!("models: none");
        return Ok(());
    }
    for (slot, assignment) in &state.model_assignments {
        println!("{slot}: {}/{}", assignment.provider_id, assignment.model_id);
    }
    Ok(())
}

fn save(ctx: &Context, name: &str) -> Result<(), CeError> {
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    let models = assignments_map(&state);
    let root = ctx.config_dir.join("profiles");
    save_profile(&root, &Profile { name: name.into(), created_at: Utc::now().to_rfc3339(), models: models.clone() })?;
    save_snapshot(&root, name, &models, &models)?;
    if !ctx.quiet {
        println!("models: profile {name} saved");
    }
    Ok(())
}

fn load(ctx: &Context, name: &str) -> Result<(), CeError> {
    let profile = load_profile(&ctx.config_dir.join("profiles"), name)?;
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    state.model_assignments.clear();
    let opencode_json = ctx.opencode_config_dir.join("opencode.json");
    for (slot, model) in &profile.models {
        let (provider_id, model_id) = model.split_once('/').ok_or_else(|| {
            CeError::Runtime(format!("profile {name} holds a malformed model {model:?}"))
        })?;
        state.set_model_assignment(slot, provider_id, model_id);
        apply_model_assignment(&opencode_json, slot, model)?;
    }
    state.save(&state_path)?;
    if !ctx.quiet {
        println!("models: profile {name} loaded");
    }
    Ok(())
}