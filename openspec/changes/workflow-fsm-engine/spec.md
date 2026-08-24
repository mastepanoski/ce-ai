# Specification: Real 7-Stage Workflow FSM Engine & Context Recovery

## Requirements

### R1: Typed 7-Stage FSM Storage & Backward Compatibility
WHEN a workflow checkpoint is saved via `ce-ai workflow checkpoint`,
THEN `ce-ai` SHALL store a `WorkflowState` object containing `stage`, `task`, `feature_name`, and `updated_at` under the `workflow` key in `state.json`. If an older `state.json` contains a legacy `last_update_check` string, `ce-ai` SHALL parse it transparently into `WorkflowState`.

### R2: Legal Transition Enforcement
WHEN transitioning between workflow stages via CLI or TUI,
THEN `ce-ai` SHALL validate the transition rule ($N \rightarrow 1$, $N \rightarrow N$, $N \rightarrow N+1$, $N \rightarrow N-1$), rejecting illegal forward jumps ($> 1$ stage) with exit code 2 (usage error).

### R3: Context Recovery Re-hydration & Fallbacks
WHEN `ce-ai workflow resume` is executed,
THEN `ce-ai` SHALL re-hydrate state from `state.workflow`, probe `openspec/changes/<feature>/` (falling back to the most recent `openspec/changes/` directory or active git branch when `feature_name` is absent), count tasks, and output structured context guidance.

### R4: Machine-Readable Export & TUI Parity
WHEN `--json` is supplied to `workflow status/checkpoint/resume`, or when selecting stages in the TUI dashboard,
THEN `ce-ai` SHALL enforce the exact same transition validation rules and state persistence models.
