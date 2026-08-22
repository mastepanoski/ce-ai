---
module: src/state/state.rs
date: '2026-08-22'
problem_type: config_error
category: config-errors
component: state_management
severity: high
symptoms:
  - models list reports none while opencode.json holds active agent.<slot> assignments
  - state.json loses model assignments while opencode.json retains agent.<slot>
root_cause: logic_error
resolution_type: code_fix
tags:
  - opencode-config
  - state-desync
  - model-assignments
  - doctor-probe
  - sync-reconciliation
  - tui-navigation
---

# Issue #111: Model Assignment Drift Reconciliation & Health Probes

## Problem

When `ce-ai` installed or updated agent slot configurations, model assignments (`agent.<slot>`) were populated in `~/.config/opencode/opencode.json`. However, if `~/.ce-ai/state.json` was reset, reinstalled, or corrupted, its `model_assignments` map was lost while `opencode.json` retained the active configuration, causing silent state desynchronization where `models list` reported `(none)` despite model assignments being present in the harness config.

## Symptoms

- Running `ce-ai models list` reported `(none)` for active agent slots even though models were configured and actively running in `opencode.json`.
- `~/.ce-ai/state.json` was missing the `model_assignments` map or contained an empty JSON object `{}` after reinstall/reset.
- `~/.config/opencode/opencode.json` retained `agent.<slot>` entries (e.g., `agent.ce-brainstorm.model = "google/gemini-2.5-flash"`).
- Inconsistent state reporting between CLI diagnostics and active runtime execution environments.

## What Didn't Work

Relying on uncoordinated state mutations where `opencode.json` and `state.json` were updated independently without validation probes or synchronization primitives:
- Direct initial configuration without defaulting `state.json` on fresh installation left state files incomplete.
- Running `ce-ai sync` only reconciled managed files (plugins and skills) while ignoring model assignment drift between configuration files.
- Lack of health checks in `ce-ai doctor` allowed desynchronized state to persist indefinitely without user visibility or corrective action guidance.

## Solution

A comprehensive multi-layered reconciliation architecture was implemented across `src/commands/install.rs`, `src/commands/doctor.rs`, `src/commands/sync.rs`, and `src/tui.rs`:

### 1. Default Assignment Initialization on Install (`src/commands/install.rs`)
During `ce-ai install`, if `state.model_assignments` is empty, `State::default_model_assignments()` is called to populate documented default assignments prior to writing state.

```rust
// If state has no model assignments yet, populate documented defaults.
if state.model_assignments.is_empty() {
    state.model_assignments = State::default_model_assignments();
}

// Apply state model assignments to target harness config.
for (slot, assignment) in &state.model_assignments {
    let model_str = format!("{}/{}", assignment.provider_id, assignment.model_id);
    let _ = crate::opencode::config::apply_model_assignment(&target_config, slot, &model_str);
}
```

### 2. Diagnostic Drift Probe in Doctor (`src/commands/doctor.rs`)
Added the `model-assignment-drift` diagnostic check to `ce-ai doctor` to detect mismatches between `opencode.json` and `state.json`.

```rust
// Model Assignment Drift Probe
if opencode_json.exists() {
    if let Ok(config_json) = read_config(&opencode_json) {
        if let Some(agents) = config_json.get("agent").and_then(|a| a.as_object()) {
            for (slot, val) in agents {
                if let Some(model_str) = val.get("model").and_then(|m| m.as_str()) {
                    if !model_str.is_empty() {
                        let state_model = state
                            .model_assignments
                            .get(slot)
                            .map(|a| format!("{}/{}", a.provider_id, a.model_id));
                        if state_model.as_deref() != Some(model_str) {
                            findings.push(format!(
                                "model-assignment-drift: slot '{slot}' configured as '{model_str}' in opencode.json but unrecorded or mismatched in state.json (run 'ce-ai sync' to reconcile)"
                            ));
                        }
                    }
                }
            }
        }
    }
}
```

### 3. Bidirectional Reconciliation in Sync (`src/commands/sync.rs`)
`ce-ai sync` now performs two-way reconciliation:
1. Reads existing `agent.<slot>.model` entries in `opencode.json` and imports missing assignments into `state.json`.
2. Applies all model assignments from `state.json` back to `opencode.json` using atomic file writes (`write_atomic`).

```rust
// Reconcile model assignments bidirectionally between opencode.json and state.json
let opencode_json = ctx.opencode_config_dir.join("opencode.json");
if opencode_json.exists() {
    if let Ok(config_json) = crate::opencode::config::read_config(&opencode_json) {
        if let Some(agents) = config_json.get("agent").and_then(|a| a.as_object()) {
            for (slot, val) in agents {
                if let Some(model_str) = val.get("model").and_then(|m| m.as_str()) {
                    if !model_str.is_empty() && !state.model_assignments.contains_key(slot) {
                        if let Some((provider, model_id)) = model_str.split_once('/') {
                            state.set_model_assignment(slot, provider, model_id);
                        }
                    }
                }
            }
        }
    }
}
for (slot, assignment) in &state.model_assignments {
    let model_str = format!("{}/{}", assignment.provider_id, assignment.model_id);
    let _ = crate::opencode::config::apply_model_assignment(&opencode_json, slot, &model_str);
}
state.save(&state_path)?;
```

### 4. Interactive Dashboard Navigation (`src/tui.rs`)
Added interactive slot navigation and status rendering in the Ratatui TUI dashboard to reflect live slot states and allow seamless model updates.

## Why This Works

- **Self-Healing State**: Bidirectional sync ensures that restoring either `state.json` or `opencode.json` automatically repairs the other file during `ce-ai sync`.
- **Early Drift Detection**: `ce-ai doctor` flags any configuration discrepancy immediately with clear remediation instructions.
- **Zero Configuration Loss**: `write_atomic` combined with JSON object merging preserves all unmanaged user configurations and plugins while modifying target agent slots.
- **Consistent Initialization**: Fresh installations always populate documented default model assignments into both state tracking and harness configuration simultaneously.

## Prevention

- **Always Pair State Writes with Health Probes**: Whenever introducing state tracked across multiple configuration files, implement a corresponding `ce-ai doctor` probe.
- **Enforce Atomic Multi-File Mutations**: Use atomic file write abstractions (`write_atomic`) to prevent partial writes during crashes or interrupted operations.
- **Provide Self-Healing Commands**: Design `sync` operations to remediate state drift automatically rather than failing or requiring manual file editing.

## Related Solutions

- [`docs/solutions/architecture/workspace-configuration-overrides-and-multi-harness-uninstall.md`](docs/solutions/architecture/workspace-configuration-overrides-and-multi-harness-uninstall.md): Establishes `.ce-ai.json` local override key-level merging over global state.
- [`docs/solutions/architecture/proactive-workflow-observability-fsm-tui-sync-watcher.md`](docs/solutions/architecture/proactive-workflow-observability-fsm-tui-sync-watcher.md): Establishes `ce-ai doctor` health probe extension patterns and TUI observability.
- [`docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md`](docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md): Defines `state.json` schema extension guidelines and `write_atomic` file integrity invariants.
