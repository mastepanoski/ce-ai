# Specification: Workflow Feature Clearing on Reset

## Requirements

### R1: Feature Clearing on Reset to Stage 1
- WHEN `validate_and_set_workflow` is called with `target_stage == WorkflowStage::Ideation` AND `current_stage != WorkflowStage::Ideation` AND `feature` is `None`, THEN `WorkflowState.feature_name` is set to `None`.

### R2: Feature Preservation on Stage Advancement and Same-Stage Checkpoints
- WHEN `validate_and_set_workflow` is called with `target_stage != WorkflowStage::Ideation` OR `current_stage == WorkflowStage::Ideation`, AND `feature` is `None`, THEN `WorkflowState.feature_name` inherits the existing `feature_name` from `current_workflow()`.

### R3: Explicit Feature Overrides
- WHEN `validate_and_set_workflow` is called with `feature == Some(non_empty_string)`, THEN `WorkflowState.feature_name` is set to `Some(trimmed_string)`.
- WHEN `validate_and_set_workflow` is called with `feature == Some("")` or whitespace only, THEN `WorkflowState.feature_name` is set to `None`.

### R4: Context Re-hydration Fallback
- WHEN `ce-ai workflow resume` probes OpenSpec context and `feature_name` is `None` (or empty), THEN `probe_openspec_context_in` falls back to discovering the most recently modified directory under `openspec/changes/`.
