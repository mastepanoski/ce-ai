# Proposal: Inferred-Stage Monotonicity Guard Hardening in State Layer

## 1. Problem Statement
In Issue #296 (PR #305, v1.40.0), Workflow FSM auto-checkpointing was introduced across 7 harnesses. While researching the edge cases, an architectural defense-in-depth gap was identified (Issue #306):

In `src/commands/workflow.rs`, the caller `maybe_auto_checkpoint` enforces that an inferred stage can never regress below the current stage:
```rust
if let Some(ref wf) = current_wf {
    if wf.source == WorkflowSource::Manual && inferred_stage.number() <= current_stage.number() {
        return Ok(None);
    }
    if inferred_stage.number() < current_stage.number() {
        return Ok(None);
    }
}
```
However, in `src/state/state.rs`, the authoritative state mutator `State::validate_and_set_workflow_for_branch` only guards against `Inferred` clobbering or regressing an existing `Manual` checkpoint:
```rust
if let Some(ref wf) = current_wf {
    if wf.source == WorkflowSource::Manual
        && source == WorkflowSource::Inferred
        && target_stage.number() <= current_stage.number()
    {
        return Ok(());
    }
}
```
Because `WorkflowStage::can_transition_to` permits a 1-stage rewind (`target_num == current_num - 1`), `validate_and_set_workflow_for_branch` by itself does not reject an `Inferred` checkpoint attempting to regress an earlier `Inferred` checkpoint.

If any future caller (e.g., batch reconciliation, a new hook handler, or a CLI command) invokes `validate_and_set_workflow_for_branch` with `WorkflowSource::Inferred` without duplicating the exact pre-check of `maybe_auto_checkpoint`, an inferred stage regression could take place.

## 2. In-Scope / Out-of-Scope Boundaries
- **In-Scope**:
  - Move the non-regression invariant for `WorkflowSource::Inferred` into `State::validate_and_set_workflow_for_branch` in `src/state/state.rs`.
  - Guarantee that when `source == WorkflowSource::Inferred`, `target_stage.number() < current_stage.number()` is unconditionally treated as a silent no-op (`Ok(())`), regardless of whether the existing checkpoint is `Manual` or `Inferred`.
  - Retain the strict guard where `wf.source == WorkflowSource::Manual && source == WorkflowSource::Inferred && target_stage.number() <= current_stage.number()` returns `Ok(())` (Inferred cannot overwrite equal Manual).
  - Add targeted unit tests in `src/state/tests/state.rs` invoking `validate_and_set_workflow_for_branch` directly to verify that Inferred-to-Inferred regressions are rejected at the state layer.
  - Bump SemVer to `1.40.1` in `Cargo.toml` and update `CHANGELOG.md`.
- **Out-of-Scope**:
  - Altering `WorkflowStage::can_transition_to` rules for `WorkflowSource::Manual` (manual operators retain the ability to rewind or reset stages).
  - Modifying the stage inference logic in `src/commands/workflow.rs`.

## 3. Risk Evaluation
- **Zero Runtime Regression**: Since `maybe_auto_checkpoint` already suppresses `inferred_stage.number() < current_stage.number()`, adding this check to `validate_and_set_workflow_for_branch` is strictly additive defense-in-depth. Existing behavior for valid transitions remains 100% identical.
- **Manual Checkpoint Freedom**: Manual checkpoints continue to enjoy valid transitions including rewinds and Stage 1 resets.

## 4. Success Criteria
- Direct invocations of `validate_and_set_workflow_for_branch` with `source: WorkflowSource::Inferred` and a target stage strictly lower than an existing `Inferred` stage result in a silent no-op (`Ok(())`), preserving the higher stage.
- All existing unit, integration, security, and E2E tests pass cleanly.
