# Exploration: Inferred-Stage Monotonicity Guard Hardening

## 1. Technical Investigation
The Workflow FSM state transitions are validated by `validate_and_set_workflow_for_branch` in `src/state/state.rs`.
Currently:
```rust
let current_wf = self.current_workflow_for_branch(root, branch);
let current_stage = current_wf
    .as_ref()
    .map(|wf| wf.stage)
    .unwrap_or(WorkflowStage::Ideation);

// Monotonic provenance guard: Inferred checkpoints can NEVER regress or clobber a Manual checkpoint at equal or higher stage
if let Some(ref wf) = current_wf {
    if wf.source == WorkflowSource::Manual
        && source == WorkflowSource::Inferred
        && target_stage.number() <= current_stage.number()
    {
        return Ok(());
    }
}

if !current_stage.can_transition_to(target_stage) {
    if source == WorkflowSource::Inferred {
        return Ok(());
    }
    return Err(CeError::Usage(...));
}
```

When `current_wf.source` is `WorkflowSource::Inferred`, and `source` is `WorkflowSource::Inferred`:
If `current_stage` is `WorkflowStage::Plan` (Stage 3), and `target_stage` is `WorkflowStage::OpenSpec` (Stage 2):
1. `wf.source == WorkflowSource::Manual` is false.
2. `current_stage.can_transition_to(target_stage)` checks:
   - `target_num == 1` -> false
   - `target_num == current_num` -> false
   - `target_num == current_num + 1` -> false
   - `current_num > 1 && target_num == current_num - 1` -> `3 > 1 && 2 == 3 - 1` -> TRUE!
3. `can_transition_to` returns `true`, and the transition is accepted!
4. Consequently, Stage 3 is overwritten by Stage 2.

In production, `src/commands/workflow.rs:maybe_auto_checkpoint` prevents this because it checks:
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
However, domain models must protect their own invariants. Leaving the non-regression invariant only in the caller violates defense-in-depth and ISO/IEC 27002 software engineering principles.

## 2. Options Evaluated

### Option A: Restrict in `WorkflowStage::can_transition_to`
Add a `source: WorkflowSource` parameter to `WorkflowStage::can_transition_to`.
- Tradeoff: `WorkflowStage` is an enum describing pure stages (1 to 7). Mixing provenance (`WorkflowSource`) into the stage representation breaks single responsibility.

### Option B: Consolidate Monotonic Guard in `State::validate_and_set_workflow_for_branch` (Selected)
Inside `validate_and_set_workflow_for_branch`:
```rust
if let Some(ref wf) = current_wf {
    if source == WorkflowSource::Inferred {
        // 1. Inferred can NEVER equal or regress an existing Manual checkpoint
        if wf.source == WorkflowSource::Manual && target_stage.number() <= current_stage.number() {
            return Ok(());
        }
        // 2. Inferred can NEVER regress an existing checkpoint (Manual or Inferred)
        if target_stage.number() < current_stage.number() {
            return Ok(());
        }
    }
}
```
- Tradeoff: Clean, localized, authoritative, and preserves pure domain boundaries.
