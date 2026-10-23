# Specification: Phase 3 — Advisory Workflow FSM & OpenSpec Decoupling (v1.77.0)

## Requirement 1: Advisory Workflow Observation Engine

### WHEN `ObservableWorkflowState::observe` is invoked on a repository
- **THEN** it MUST inspect git branch, working tree diffs, markdown plans, review receipts, and solution docs directly from the filesystem without mutating `state.json`.
- **THEN** it MUST return `active_work: true` when uncommitted changes exist in the working directory or an active non-main branch has changes.
- **THEN** it MUST return `plan: Some(PlanObservation)` when a markdown plan exists in `CeDocsConfig::plans_dir`, correctly calculating completed `- [x]` and total items.
- **THEN** it MUST return `openspec: None` when no active change package exists under `openspec/changes/`, treating OpenSpec as completely optional and non-blocking.

## Requirement 2: Decoupled OpenSpec & Non-Blocking Gate

### WHEN `ce-ai gate check` is executed without explicit `--mode` or `CE_AI_GATE_MODE`
- **THEN** `resolve_gate_mode` MUST resolve to `GateMode::Observe`.
- **THEN** the gate check MUST NOT block code writes (`GateDecision::Blocked`) due to the absence of `openspec/changes/` artifacts.
- **THEN** the process MUST exit with code `0` (Success) in default configuration, recording advisory telemetry to `gate-events.jsonl`.

### WHEN an agent performs a write on `src/**` on a branch without an OpenSpec package
- **THEN** `evaluate_gate_policy` in default observe mode MUST NOT block the write.
- **THEN** the policy decision MUST be `Pass` or `WouldBlock` (advisory only), never preventing file modification.

## Requirement 3: Advisory Capabilities Matrix in Workflow Status

### WHEN `ce-ai workflow status` is executed
- **THEN** it MUST display the `Advisory Workflow Capabilities Matrix` reflecting real-time repository observations.
- **THEN** it MUST clearly label OpenSpec as optional / inactive when no change folder exists.
- **THEN** it MUST not fail or warn when an agent writes code without an OpenSpec package.

### WHEN `ce-ai workflow status --json` is executed
- **THEN** the serialized JSON output MUST include the structured `observable_state` object containing `active_work`, `active_branch`, `plan`, `verification`, `handoff`, `knowledge_capture`, and `openspec`.

## Requirement 4: Principle of No Semantic Authority in Governance

### WHEN AI agents inspect `AGENTS.md` operating directives
- **THEN** Invariant 6 MUST explicitly declare the "No Semantic Authority" rule: CE-AI does not impose mandatory stages or blocking write gates beyond Compound Engineering contracts, and OpenSpec is decoupled as an optional integration.
