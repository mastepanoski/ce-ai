//! `ce-ai models`: persist model assignments in state.json (MM-1), apply them
//! to opencode.json `agent.<slot>.model` (MM-2), and manage named profiles
//! with append-only snapshots (MM-3, MM-4). Model assignment is always
//! user-driven; ce-ai never seeds default models (#111).

use std::collections::BTreeMap;

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::config::apply_model_assignment;
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

/// Discovers the models offered by the active opencode installation by
/// querying its CLI (`opencode models`), so pickers reflect what the
/// configured providers actually offer instead of a hardcoded list (#111).
pub fn discover_models() -> Result<Vec<String>, CeError> {
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
    /// Agent slot, e.g. ce-brainstorm.
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
        .map(|(slot, assignment)| {
            (
                slot.clone(),
                format!("{}/{}", assignment.provider_id, assignment.model_id),
            )
        })
        .collect()
}

pub(crate) fn set(ctx: &Context, slot: &str, model: &str) -> Result<(), CeError> {
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
    if !config
        .get("agent")
        .and_then(|agents| agents.get(slot))
        .is_some_and(serde_json::Value::is_object)
    {
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
            // Split validated above (contains '/').
            let (provider_id, model_id) = model.split_once('/').expect("validated split");
            state.set_model_assignment(slot, provider_id, model_id);
            imported.push((slot.clone(), model.to_string()));
        }
    }
    imported
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::harness::agents::ORCHESTRATOR_AGENT as ORCHESTRATOR_SLOT;
    use tempfile::TempDir;

    /// Builds a fully hermetic Context pointing at temp dirs; avoids
    /// `Context::resolve` so tests never touch the host opencode.json.
    fn hermetic_ctx(tmp: &TempDir) -> Context {
        Context {
            config_dir: tmp.path().join("ce-ai"),
            opencode_config_dir: tmp.path().join("home/.config/opencode"),
            dry_run: false,
            verbose: false,
            quiet: true,
        }
    }

    #[test]
    fn syncs_across_all_active_harnesses() {
        let tmp = TempDir::new().unwrap();
        let ctx = hermetic_ctx(&tmp);
        std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
        std::fs::write(
            ctx.opencode_config_dir.join("opencode.json"),
            r#"{"plugin":["user"]}"#,
        )
        .unwrap();

        set(&ctx, "ce-brainstorm", "anthropic/claude-3-5-sonnet").unwrap();
        let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
        assert_eq!(state.model_assignments.len(), 1);
        assert_eq!(
            state
                .model_assignments
                .get("ce-brainstorm")
                .unwrap()
                .model_id,
            "claude-3-5-sonnet"
        );
    }

    #[test]
    fn parse_models_output_extracts_provider_model_tokens() {
        let text = "\
opencode/big-pickle
opencode-go/kimi-k2.6  (recommended)
anthropic/claude-sonnet-4-5   vision tools

not-a-model-line
/bad
also/bad/
";
        assert_eq!(
            parse_models_output(text),
            vec![
                "anthropic/claude-sonnet-4-5".to_string(),
                "opencode-go/kimi-k2.6".to_string(),
                "opencode/big-pickle".to_string(),
            ]
        );
    }

    #[test]
    fn drift_findings_detect_divergent_and_missing_slots() {
        let mut state = State::new();
        state.set_model_assignment("ce-brainstorm", "anthropic", "claude-x");
        let config = serde_json::json!({
            "agent": {
                "ce-brainstorm": { "model": "user/custom-model" },
                ORCHESTRATOR_SLOT: { "model": "opencode-go/kimi-k2.6" },
                "third-party": { "model": "vendor/model" }
            }
        });
        let findings = model_drift_findings(&state, &config);
        assert!(findings
            .iter()
            .any(|f| f.contains("'ce-brainstorm'") && f.contains("config='user/custom-model'")));
        assert!(findings
            .iter()
            .any(|f| f.contains(ORCHESTRATOR_SLOT) && f.contains("untracked in state.json")));
        assert!(!findings.iter().any(|f| f.contains("third-party")));
    }

    #[test]
    fn drift_findings_detect_state_slot_missing_from_config() {
        let mut state = State::new();
        state.set_model_assignment("ce-plan", "opencode-go", "kimi-k2.6");
        let findings = model_drift_findings(&state, &serde_json::json!({}));
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("missing from opencode.json agent map"));
    }

    #[test]
    fn import_config_assignments_repairs_desync() {
        let mut state = State::new();
        state.set_model_assignment("ce-plan", "old", "model");
        let config = serde_json::json!({
            "agent": {
                "ce-plan": { "model": "new/model" },
                "custom-slot": { "model": "user/model" },
                "broken-slot": { "model": "no-slash" }
            }
        });
        let imported = import_config_assignments(&mut state, &config);
        assert_eq!(
            imported,
            vec![
                ("ce-plan".to_string(), "new/model".to_string()),
                ("custom-slot".to_string(), "user/model".to_string()),
            ]
        );
        assert_eq!(state.model_assignments["ce-plan"].provider_id, "new");
        // Re-import is a no-op once state matches config.
        assert!(import_config_assignments(&mut state, &config).is_empty());
    }
}
