# Proposal: Phase 3 — Advisory Workflow FSM & OpenSpec Decoupling (v1.77.0)

## Problem Statement

Historically, `ce-ai` operated as an authoritative workflow gatekeeper. It tracked a rigid 7-stage cursor (`WorkflowStage`) in `state.json` and blocked agent code writes in Stage 4 (`ce-work`) via `ce-ai gate check` unless formal OpenSpec documents (`proposal.md`, `spec.md`, `tasks.md`) existed under `openspec/changes/<feature>/`.

As highlighted in the upstream Compound Engineering maintainer review:
1. **Repository Reality as SSOT:** In Compound Engineering, the plan file, the branch, the commits, and the PR *are* the workflow state. A separate state cursor stored in `state.json` invariably drifts from reality.
2. **False Linearity & Non-Linear Workflows:** Real-world engineering is not strictly linear: trivial changes skip formal plans, `ce-debug` follows an independent diagnosis cycle, and `ce-compound` writes documentation only when novel domain learnings arise.
3. **Artificial Documentation Debt:** Forcing formal OpenSpec specs for every change unit causes documentation bloat and cognitive friction, inflating the volume of documents that must be kept trustworthy.
4. **Principle of No Semantic Authority:** `ce-ai` must not impose mandatory workflow stages, artifacts, or blocking write gates beyond those defined by Compound Engineering itself.

Phase 3 transitions `ce-ai`'s workflow system from an authoritative gatekeeper to a read-only, **Advisory Workflow Observation Engine**, decoupling OpenSpec into an optional, opt-in capability.

## Proposed Changes

1. **Advisory Workflow Observation Subsystem (`src/observation/`):**
   - Introduce `ObservableWorkflowState` deriving real-time state from repository artifacts:
     - `active_work`: detected via uncommitted git changes and branch context.
     - `plan`: detected via `.md` plan files in `CeDocsConfig::plans_dir`.
     - `verification`: detected via review receipts and test execution markers.
     - `handoff`: detected via handoff receipts or documentation artifacts.
     - `knowledge_capture`: evaluated against recent changes and solution documentation.
     - `openspec`: detected optionally if `openspec/changes/<feature>` exists; returns `None` if inactive.
2. **Decouple OpenSpec from Core Workflow:**
   - Remove OpenSpec from the core domain invariants.
   - OpenSpec becomes an optional, opt-in workflow integration for high-complexity features or regulated systems.
3. **Advisory Gate Check Engine:**
   - Shift default gate mode from `Enforce` (blocking) to `Observe` (advisory).
   - In `evaluate_gate_policy`, never block agent writes when OpenSpec contracts are absent unless explicitly configured in legacy enforcement mode.
   - Record telemetry to `gate-events.jsonl` without halting tool execution.
4. **Non-Blocking `ce-ai workflow status`:**
   - Enhance `ce-ai workflow status` and `ce-ai workflow status --json` to present the Observable Capabilities Matrix alongside advisory phase recommendations.
5. **Operating Directive Update (`AGENTS.md`):**
   - Formalize the "No Semantic Authority" invariant, replacing the mandatory OpenSpec gate with the optional OpenSpec model.

## In-Scope vs. Out-of-Scope

### In-Scope
- `src/observation/mod.rs` and `src/observation/state.rs` implementation with comprehensive unit tests.
- `src/commands/gate.rs`: Defaulting to `GateMode::Observe`, eliminating blocking writes on missing specs.
- `src/commands/workflow.rs`: Integrating `ObservableWorkflowState` into status lines and JSON output.
- Updating `state.json` serialization and loading to treat `WorkflowState` as an advisory snapshot.
- Updating `AGENTS.md` and accreting domain concepts in `CONCEPTS.md`.
- Full verification gates (`cargo fmt`, `cargo clippy`, `cargo test`).

### Out-of-Scope
- Decommissioning tarball scraping and file-level restoration (scheduled for Phase 4 / v2.0 GA).
- Full removal of `WorkflowStage` enum (retained for backwards-compatible CLI checkpointing and serialization).

## Success Criteria

1. Agents can execute tool writes without being blocked by missing `openspec/changes/` artifacts.
2. `ce-ai workflow status` cleanly displays the Observable Capabilities Matrix derived directly from repository artifacts.
3. `ce-ai gate check` defaults to advisory observation, logging write telemetry without exiting with non-zero error codes.
4. All unit, CLI, and integration tests pass with 100% green matrix.
