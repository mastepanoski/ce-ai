# Design: Architecture and Implementation Contracts for Mid-Tier Slot

## 1. Constants and Predicates (`src/harness/agents.rs`)

```rust
/// Agent slot name reserved for the Compound Engineering orchestrator.
pub const ORCHESTRATOR_AGENT: &str = "ce-ai";

/// Slot name for ce-code-review's mid-tier persona dispatch.
pub const CODE_REVIEW_MID_TIER_SLOT: &str = "ce-code-review-mid-tier";

/// Primary CE workflow stage slots.
pub const CE_AGENT_STAGE_SLOTS: [&str; 6] = [
    ORCHESTRATOR_AGENT,
    "ce-brainstorm",
    "ce-plan",
    "ce-work",
    "ce-code-review",
    "ce-doc-review",
];

/// All CE workflow and tiering slots tracked across harnesses.
pub const CE_AGENT_SLOTS: [&str; 7] = [
    ORCHESTRATOR_AGENT,
    "ce-brainstorm",
    "ce-plan",
    "ce-work",
    "ce-code-review",
    "ce-doc-review",
    CODE_REVIEW_MID_TIER_SLOT,
];

/// Checks if a slot is an internal persona/tiering slot rather than a top-level stage.
pub fn is_tier_slot(slot: &str) -> bool {
    slot == CODE_REVIEW_MID_TIER_SLOT
}

/// Checks if a slot is a primary workflow stage slot.
pub fn is_stage_slot(slot: &str) -> bool {
    CE_AGENT_STAGE_SLOTS.contains(&slot)
}
```

## 2. Formatting Model Assignments (`src/commands/models.rs`)

To ensure `ce-ai models list` presents the mid-tier slot as a distinguished sub-slot rather than an independent stage:

```rust
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
        lines.push(format!("{slot}: {}/{}", assignment.provider_id, assignment.model_id));
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
    if mid_tier_assignment.is_some() && !assignments.contains_key("ce-code-review") {
        let sub = mid_tier_assignment.unwrap();
        lines.push(format!(
            "{mid_tier_slot} (mid-tier sub-slot): {}/{}",
            sub.provider_id, sub.model_id
        ));
    }

    lines
}
```

`list(ctx)` simply iterates over the result of `format_model_assignments(&state.model_assignments)` and prints each line.

## 3. Diagnostic Note Helper (`src/commands/models.rs`)

```rust
/// Evaluates whether an informational note should be emitted regarding unconfigured
/// mid-tier persona dispatch for `ce-code-review`.
pub fn check_code_review_mid_tier_note(
    state: &crate::state::state::State,
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
```

## 4. Doctor Integration (`src/commands/doctor.rs`)

In `src/commands/doctor.rs` at the opencode model check block (lines 112-117):
```rust
    // Model assignment drift between state.json and opencode.json (#111).
    if let Ok(config) = read_config(&opencode_json) {
        findings.extend(crate::commands::models::model_drift_findings(
            &state, &config,
        ));
        if let Some(note) = crate::commands::models::check_code_review_mid_tier_note(&state, &config) {
            println!("doctor-info: {note}");
        }
    }
```
Because the note is printed with `println!("doctor-info: {note}")` and not added to `findings`, `ce-ai doctor` does not exit with a failure code when this condition triggers.
