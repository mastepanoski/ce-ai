//! Orchestrator agent definitions seeded per harness (model-defaults-tui-orchestrator).
//!
//! ce-ai creates the STRUCTURAL agent entry (`ce-ai`) so users get a ready
//! orchestrator, but never assigns `model`/`variant` — that customization
//! belongs to the user via `ce-ai models set`, the TUI picker, or direct
//! config edits (#111).

use std::path::Path;

use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::state::write_atomic;

/// Agent slot name reserved for the Compound Engineering orchestrator.
pub const ORCHESTRATOR_AGENT: &str = "ce-ai";

/// CE workflow slots whose configuration ce-ai tracks across harnesses.
pub const CE_AGENT_SLOTS: [&str; 6] = [
    ORCHESTRATOR_AGENT,
    "ce-brainstorm",
    "ce-plan",
    "ce-work",
    "ce-code-review",
    "ce-doc-review",
];

/// Description advertised for the orchestrator agent.
pub const ORCHESTRATOR_DESCRIPTION: &str = "CE AI Orchestrator - coordinates compound engineering";

/// Harnesses whose config format supports structured agent entries (Opencode JSON schema).
pub fn supports_agent_definitions(harness: &HarnessKind) -> bool {
    matches!(harness, HarnessKind::Opencode)
}

/// Applies a user-driven model assignment to `agent.<slot>.model` in any
/// agent-capable harness config, preserving every other key on the slot and
/// every other agent entry. Never writes `variant` (#111). Hard-fails on a
/// malformed `agent` map instead of clobbering it (D4).
pub fn apply_agent_model(config_path: &Path, slot: &str, model_value: &str) -> Result<(), CeError> {
    let mut config = crate::state::read_config(config_path)?;
    let agents = match config.get_mut("agent") {
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => {
            return Err(CeError::Runtime(
                "`agent` in harness config must be an object; refusing to overwrite it. Fix the file manually, then re-run."
                    .into(),
            ))
        }
        None => {
            config["agent"] = serde_json::json!({});
            config["agent"].as_object_mut().ok_or_else(|| {
                CeError::Runtime("failed to initialize agent object in harness config".into())
            })?
        }
    };
    let entry = agents
        .entry(slot.to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !entry.is_object() {
        return Err(CeError::Runtime(format!(
            "`agent.{slot}` in harness config must be an object; refusing to overwrite it. Fix the file manually, then re-run."
        )));
    }
    entry["model"] = serde_json::Value::String(model_value.to_string());
    write_atomic(config_path, &serde_json::to_vec_pretty(&config)?)
}

/// Ensures the structural `ce-ai` orchestrator agent exists in the target
/// harness config. Idempotent: an existing `agent.ce-ai` entry is preserved
/// verbatim (including any user-assigned model). Never writes `model` or
/// `variant`. Returns true when the entry was created.
///
/// Markdown-based harness configs (cursor/copilot rules files) have no
/// agent-map concept; callers get `Ok(false)` for those instead of a
/// fabricated structure.
pub fn ensure_orchestrator_agent(
    config_path: &Path,
    harness: &HarnessKind,
) -> Result<bool, CeError> {
    if !supports_agent_definitions(harness) {
        return Ok(false);
    }
    let mut config = crate::state::read_config(config_path)?;
    let agents = match config.get_mut("agent") {
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => {
            return Err(CeError::Runtime(
                "`agent` in harness config must be an object; refusing to overwrite it. Fix the file manually, then re-run."
                    .into(),
            ))
        }
        None => {
            config["agent"] = serde_json::json!({});
            config["agent"].as_object_mut().ok_or_else(|| {
                CeError::Runtime("failed to initialize agent object in harness config".into())
            })?
        }
    };
    if agents
        .get(ORCHESTRATOR_AGENT)
        .is_some_and(|v| v.is_object())
    {
        return Ok(false);
    }
    agents.insert(
        ORCHESTRATOR_AGENT.to_string(),
        serde_json::json!({
            "description": ORCHESTRATOR_DESCRIPTION,
            "mode": "primary",
            "permission": {
                "question": "allow",
                "task": { "*": "allow" }
            }
        }),
    );
    write_atomic(config_path, &serde_json::to_vec_pretty(&config)?)?;
    Ok(true)
}

#[cfg(test)]
#[path = "tests/agents.rs"]
mod tests;
