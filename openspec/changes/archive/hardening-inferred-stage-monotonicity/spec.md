# Spec: Inferred-Stage Monotonicity Guard Hardening

## Requirements

### Requirement 1: Non-Regression of Inferred Checkpoints in State Layer
WHEN `validate_and_set_workflow_for_branch` is called with `source = WorkflowSource::Inferred`
AND the workspace/branch already has an existing workflow at `current_stage`
AND `target_stage.number() < current_stage.number()`
THEN the function SHALL return `Ok(())` without modifying the existing workflow state in `self.workflows` or `self.workflow`.

### Requirement 2: Preservation of Manual Provenance Guard
WHEN `validate_and_set_workflow_for_branch` is called with `source = WorkflowSource::Inferred`
AND the workspace/branch has an existing workflow with `source = WorkflowSource::Manual`
AND `target_stage.number() <= current_stage.number()`
THEN the function SHALL return `Ok(())` without modifying the existing workflow state.

### Requirement 3: Allowance of Manual Rewinds
WHEN `validate_and_set_workflow_for_branch` is called with `source = WorkflowSource::Manual`
AND `target_stage` is a legal rewind (`target_num == current_num - 1` or `target_num == 1`)
THEN the transition SHALL be accepted and recorded with `source = WorkflowSource::Manual`.
