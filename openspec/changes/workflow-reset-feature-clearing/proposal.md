# Proposal: Reset Workflow Feature Clearing

## Problem Statement
When resetting the 7-stage workflow FSM to Stage 1 (Ideation) from an advanced stage (e.g. Stage 2+), `validate_and_set_workflow` unconditionally inherits the previous workflow's `feature_name` if `--feature` is omitted. Consequently, subsequent `workflow resume` invocations re-hydrate stale context from the previous feature instead of falling back to the active repository state. Furthermore, passing an empty string `--feature ""` currently populates `Some("")`, causing downstream `probe_openspec_context_in` path resolution to target `openspec/changes/` directly rather than executing directory discovery.

## In-Scope
- Modify `validate_and_set_workflow` in `src/state/state.rs` so that transitioning to Stage 1 from a different stage defaults `feature_name` to `None` when `--feature` is omitted.
- Treat empty or whitespace-only feature values (`--feature ""`) as explicit clears (`None`) across all stages.
- Retain inheritance of `feature_name` across advancing (N -> N+1) or same-stage transitions when `--feature` is omitted.
- Strengthen `probe_openspec_context_in` in `src/commands/workflow.rs` with defense-in-depth non-empty filtering.
- Comprehensive unit and CLI integration tests validating feature clearing, retention, and explicit override behaviors.

## Out-of-Scope
- Altering the 7-stage FSM transition rules or state serialization schema.
- Retroactive modifications to historical brainstorm artifacts (`docs/brainstorms/`).

## Risk Evaluation
- **Low Risk:** Localized to `feature_name` resolution in `validate_and_set_workflow` and context probing.
- **Backward Compatibility:** Preserved. Existing workflows advancing between stages without `--feature` continue inheriting seamlessly.

## Success Criteria
- Resetting to Stage 1 without `--feature` clears the prior feature from `state.json`.
- Advancing stages without `--feature` retains the active feature.
- Explicit `--feature <name>` always sets the specified feature.
- Explicit `--feature ""` clears the active feature.
- All unit and CLI integration tests pass.
