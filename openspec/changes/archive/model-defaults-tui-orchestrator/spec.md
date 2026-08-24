# Spec: model-defaults-tui-orchestrator

## Requirement: Default model assignments with orchestrator slot `ce-ai`

The system SHALL define default model assignments covering the orchestrator slot named `ce-ai` and the per-stage slots (`ce-brainstorm`, `ce-plan`, `ce-work`).

### Scenario: Fresh install seeds defaults
- **WHEN** `ce-ai install` completes against an `opencode.json` whose agent map lacks models for the default slots
- **THEN** each default slot (including `ce-ai`) receives its documented default `provider/model` in both `opencode.json` and `state.json`

### Scenario: Existing user assignments are preserved
- **WHEN** a default slot already has a non-empty `model` in `opencode.json` or an assignment in `state.json`
- **THEN** install MUST NOT change that slot's value
- **AND** the remaining unset slots are still seeded

### Scenario: Dry-run writes nothing
- **WHEN** install runs with dry-run enabled
- **THEN** no defaults are written to any file

## Requirement: TUI model customization backed by live harness catalog

The TUI Models tab SHALL allow selecting an agent slot and assigning it a model discovered from the active harness (its CLI), without leaving the dashboard.

### Scenario: Pick and apply a discovered model from the TUI
- **WHEN** the user selects a slot, opens the picker, chooses an entry from the harness-provided catalog, and confirms
- **THEN** the assignment is applied through the same atomic path as `ce-ai models set`
- **AND** both `opencode.json` and `state.json` reflect the new model after the modal closes

### Scenario: Discovery failure is explicit
- **WHEN** the harness CLI is unavailable, exits non-zero, or returns no usable entries
- **THEN** the picker shows an explicit error modal
- **AND** no static fallback list is presented and no file is modified

### Scenario: Cancel picker
- **WHEN** the user closes the picker without confirming
- **THEN** no file is modified

## Requirement: State/config drift detection

`ce-ai doctor` SHALL report mismatches between `state.json` model assignments and `opencode.json` agent models.

### Scenario: Config-only assignment detected
- **WHEN** a CE-known slot has a `model` in `opencode.json` but no entry in `state.json`
- **THEN** doctor reports a `models-drift` finding and exits non-zero

### Scenario: Divergent values detected
- **WHEN** state and config disagree on the model for a tracked slot
- **THEN** doctor reports the finding showing both values

### Scenario: Third-party slots ignored
- **WHEN** opencode.json contains unknown agent slots with models but no state entries
- **THEN** doctor MUST NOT report them

## Requirement: Desync repair on sync

`ce-ai sync` SHALL import effective `opencode.json` model assignments into `state.json`.

### Scenario: Import repairs desync
- **WHEN** sync runs and config holds assignments absent or divergent in state
- **THEN** state.json is updated to match config for those slots
- **AND** opencode.json is not modified by the import

## Acceptance criteria

- All scenarios above covered by unit tests.
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` pass.
