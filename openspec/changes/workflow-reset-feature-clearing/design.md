# Design: Workflow Feature Clearing on Reset

## Technical Design

### 1. `validate_and_set_workflow` Resolution Logic (`src/state/state.rs`)

```rust
let is_reset_to_stage_1 = target_stage == WorkflowStage::Ideation && current_stage != WorkflowStage::Ideation;

let feature_name = match feature {
    Some(f) => {
        let trimmed = f.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }
    None => {
        if is_reset_to_stage_1 {
            None
        } else {
            self.current_workflow().and_then(|wf| wf.feature_name)
        }
    }
};
```

### 2. Context Probing Defense-in-Depth (`src/commands/workflow.rs`)
In `probe_openspec_context_in`:
```rust
let target_feature = if let Some(feat) = wf
    .as_ref()
    .and_then(|w| w.feature_name.clone())
    .filter(|f| !f.trim().is_empty())
{
    feat
} else {
    // Fallback to most recently modified directory in openspec/changes/
    ...
};
```

### 3. CLI Argument Guidance Alignment (`src/commands/workflow.rs`)
Update `Action::Checkpoint.stage` doc comment from:
`/// Current 7-stage phase (e.g. "4" or "work" or "Stage 4: TDD & Work").`
to:
`/// Current 7-stage phase (e.g. "4", "work", "tdd").`
