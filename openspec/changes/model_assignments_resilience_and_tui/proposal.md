# Proposal: Model Assignments Resilience, Drift Reconciliation, and TUI Editor

## Problem Statement
Currently, `ce-ai` writes `agent.<slot>` entries into `~/.config/opencode/opencode.json` when setting model assignments (e.g. `ce-brainstorm`). However, if `~/.ce-ai/state.json` is missing, corrupted, or reset during reinstall/resync operations, `state.json` loses its `model_assignments` map while `opencode.json` retains the `agent.<slot>` assignments.

This creates two critical flaws:
1. **Silent State Drift**: `ce-ai models list` reports `(none)` while `opencode.json` still carries active model assignments.
2. **Lack of Diagnostics & Recovery**: `ce-ai doctor` does not detect model assignment drift, and `ce-ai sync` does not repair it. Furthermore, default model assignments are not populated on fresh installs, and the TUI Models tab is strictly read-only.

## In-Scope
1. **Model Assignment Health Probe in `ce-ai doctor`**: Detect mismatches between `state.json` model assignments and active harness configurations (`opencode.json`).
2. **Reconciliation in `ce-ai sync`**: Automatically import missing assignments from harness configs into `state.json` or sync state assignments back to harness configs.
3. **Default Model Assignments in `ce-ai install`**: Apply documented default model assignments for `ce-ai` (orchestrator slot) and stage slots (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, `ce-doc-review`) on fresh installs without overwriting existing user choices.
4. **Interactive TUI Model Assignment Editor**: Enable interactive navigation, editing, and saving of model slot assignments directly inside the Ratatui TUI dashboard.

## Out-of-Scope
- Querying live online LLM provider APIs for real-time pricing or billing data.

## Risk Evaluation & Mitigation
- **Risk**: Overwriting user-customized `agent.<slot>` configurations in `opencode.json`.
- **Mitigation**: All updates must use `crate::state::write_atomic` and parse existing JSON structures to preserve unmanaged user keys, custom plugins, and custom skills.

## Success Criteria
- `ce-ai doctor` flags desynchronized model assignments as diagnostic findings.
- `ce-ai sync` repairs model assignment drift bidirectionally without data loss.
- TUI Models tab allows selecting and updating model slots interactively.
- All unit, CLI, and security tests pass cleanly (`cargo test` & `make e2e`).
