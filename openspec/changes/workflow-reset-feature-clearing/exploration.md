# Exploration: Workflow Feature Clearing on Reset

## Investigation of Current Behavior
In `src/state/state.rs`:
```rust
let feature_name = feature.or_else(|| self.current_workflow().and_then(|wf| wf.feature_name));
```
When `feature` is `None`, `.or_else()` unconditionally reuses the previous `WorkflowState.feature_name`.
When resetting to Stage 1 via `ce-ai workflow checkpoint --stage 1 --task "ideation for feature-b"`, the user is starting a brand new cycle where the feature may not yet exist in `openspec/changes/`.
Because the old feature name is retained:
1. `state.workflow` records the old feature name.
2. `ce-ai workflow resume` executes `probe_openspec_context_in`, reads `Some("old-feature")`, and re-hydrates against the wrong OpenSpec directory.
3. The automatic fallback (discovering the most recently modified feature directory) is bypassed.

## Evaluated Options

### Option A: Unconditional clearing on all backward transitions
- **Pros:** Guarantees no stale feature on rewind.
- **Cons:** Too aggressive. If a user temporarily rewinds from Stage 4 (Work) to Stage 3 (Plan) to revise task breakdown for the *same* feature without passing `--feature`, they would unexpectedly lose their feature association.

### Option B: Reset-to-Stage-1 clearing + explicit empty string clearing (Chosen)
- **Logic:**
  1. If `target_stage == WorkflowStage::Ideation` AND `current_stage != WorkflowStage::Ideation`, and no explicit `feature` is provided, do NOT inherit; set `feature_name = None`.
  2. If an explicit non-empty `feature` is provided (`Some(name)` where `!name.trim().is_empty()`), set `feature_name = Some(name.trim())`.
  3. If an explicit empty `feature` is provided (`Some("")` or whitespace), treat as an intentional clear: `feature_name = None`.
  4. In all other transitions (advancing N -> N+1, staying in N, or minor rewind within an active feature), inherit `current_workflow().and_then(|wf| wf.feature_name)` if `--feature` was omitted.
- **Pros:** Perfectly models developer expectations. Starting fresh at Stage 1 resets feature context; working within an existing feature retains it effortlessly; explicit `--feature ""` allows clearing anywhere.

## Architectural Tradeoffs
- Zero changes to storage schema in `state.json`.
- Minimal cognitive load: users only specify `--feature` when establishing or explicitly changing the feature name.
