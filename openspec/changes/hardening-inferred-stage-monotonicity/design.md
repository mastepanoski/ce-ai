# Design: Inferred-Stage Monotonicity Guard Hardening in State Layer

## 1. Architectural Overview
The state layer (`src/state/state.rs`) acts as the single source of truth for workflow state mutations across workspaces and branches.

### Target Invariant Matrix
| Existing State Source | Target Transition Source | Condition | Action |
|---|---|---|---|
| `Manual` (Stage N) | `Inferred` (Stage M) | `M <= N` | Silent no-op (`Ok(())`) |
| `Manual` (Stage N) | `Inferred` (Stage M) | `M > N` && legal transition | Accepted (`stage = M`) |
| `Inferred` (Stage N) | `Inferred` (Stage M) | `M < N` | Silent no-op (`Ok(())`) |
| `Inferred` (Stage N) | `Inferred` (Stage M) | `M == N` | Accepted (updates timestamp/metadata) |
| `Inferred` (Stage N) | `Inferred` (Stage M) | `M > N` && legal transition | Accepted (`stage = M`) |
| Any (Stage N) | `Manual` (Stage M) | legal transition | Accepted (`stage = M`) |
| Any (Stage N) | `Manual` (Stage M) | illegal transition | Error (`CeError::Usage`) |

## 2. Implementation Details in `src/state/state.rs`
Update `State::validate_and_set_workflow_for_branch`:
```rust
// Monotonic provenance guard:
// 1. Inferred checkpoints can NEVER regress or clobber a Manual checkpoint at equal or higher stage.
// 2. Inferred checkpoints can NEVER regress an existing checkpoint (Manual or Inferred).
if let Some(ref wf) = current_wf {
    if source == WorkflowSource::Inferred {
        if wf.source == WorkflowSource::Manual && target_stage.number() <= current_stage.number() {
            return Ok(());
        }
        if target_stage.number() < current_stage.number() {
            return Ok(());
        }
    }
}
```

## 3. Unit Test Design (`src/state/tests/state.rs`)
Add unit test: `inferred_checkpoint_cannot_regress_previous_inferred_checkpoint`:
1. Initialize `State::new()`.
2. Set workflow with `WorkflowSource::Inferred` to `WorkflowStage::Plan` (Stage 3).
3. Attempt to set workflow with `WorkflowSource::Inferred` to `WorkflowStage::OpenSpec` (Stage 2).
4. Assert return value is `Ok(())`.
5. Assert current workflow stage remains `WorkflowStage::Plan` (Stage 3).
6. Attempt to set workflow with `WorkflowSource::Inferred` to `WorkflowStage::WorkTdd` (Stage 4).
7. Assert return value is `Ok(())`.
8. Assert current workflow stage advances to `WorkflowStage::WorkTdd` (Stage 4).
