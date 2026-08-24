# Design: model-defaults-tui-orchestrator

## Data

```rust
// src/commands/models.rs
pub const ORCHESTRATOR_SLOT: &str = "ce-ai";

/// (slot, provider/model) applied by install when the slot has no model yet.
pub const DEFAULT_MODEL_ASSIGNMENTS: [(&str, &str); 4] = [
    ("ce-ai", "opencode-go/kimi-k2.6"),        // orchestrator
    ("ce-brainstorm", "opencode-go/kimi-k2.6"),
    ("ce-plan", "opencode-go/kimi-k2.6"),
    ("ce-work", "opencode-go/kimi-k2.6"),
];
```

No static model catalog: the TUI picker discovers models at runtime from the
active harness CLI (`opencode models`), so it always reflects what the
configured providers actually offer.

## Interfaces

### models.rs

- `pub(crate) fn set(ctx, slot, model)` — visibility widened for TUI reuse (same behavior, unchanged).
- `pub fn apply_defaults(ctx: &Context) -> Result<Vec<(String, String)>, CeError>`
  - For each `(slot, model)` in `DEFAULT_MODEL_ASSIGNMENTS`:
    - Skip if `opencode.json → agent.<slot>.model` is a non-empty string, **or** state already holds an assignment for the slot.
    - Otherwise call `set(ctx, slot, model)`.
  - Returns seeded `(slot, model)` pairs (for install logging); never mutates pre-existing slots.
- `fn parse_models_output(text: &str) -> Vec<String>` — pure parser: trims lines,
  keeps the first whitespace-delimited token when it matches `provider/model`
  with non-empty segments and a single slash; sorted + deduped. Unit-tested.
- `pub fn discover_models() -> Result<Vec<String>, CeError>` — runs
  `opencode models`, hard-fails on spawn failure / non-zero exit / empty result
  (no silent fallbacks); returns parsed catalog.
- `pub fn model_drift_findings(state: &State, config: &serde_json::Value) -> Vec<String>`
  - `models-drift: slot '<s>' config='<m>' state='<m2>'` — both present, different.
  - `models-drift: slot '<s>' missing from opencode.json agent map` — state-only.
  - `models-drift: slot '<s>' present in opencode.json but untracked in state.json` — config-only, only for CE-known slots (`ORCHESTRATOR_SLOT` + defaults).
- `pub fn import_config_assignments(state: &mut State, config: &serde_json::Value) -> Vec<(String, String)>`
  - Imports every config assignment that is absent or divergent in state;
    skips malformed values (no `/`). Pure w.r.t. filesystem.

### install.rs

- After `state.save`, when `!ctx.dry_run`: call `models::apply_defaults(ctx)` and print `install: default model <slot> = <model>` per seeded pair.

### doctor.rs

- Pushes `models::model_drift_findings(&state, &config)` into the existing findings vec (same exit semantics). Invalid config is already reported separately; drift check treats it as empty.

### sync.rs

- In `sync_with`, before final `state.save`: read opencode.json, run `import_config_assignments`, print `sync: imported model <slot> = <model>` per change.

### tui.rs

- `App` gains: `model_slots: Vec<String>` (defaults + tracked union), `selected_model_idx`, `model_picker_open`, `picker_items`, `picker_selected`.
- Key handling order: output modal close → picker navigation (Up/Down/j/k, Enter applies via `models::set`, Esc cancels) → Models-tab keys (`n`/`p` cursor, `m` opens picker via `discover_models`) → global keys.
- Picker modal renders the discovered catalog with selection highlight; discovery errors surface in an explicit error modal.
- Models tab renders the slot list with a selection cursor, current value or `(unset)`, and key hints.

## Invariants honored

- All writes flow through `write_atomic` via existing `set`/`apply_model_assignment`/`state.save`.
- Defaults never overwrite existing user models (skip condition above).
- No dummy fallbacks: picker shows an error instead of inventing a static list.
- No new exit codes; doctor reuses its findings→non-zero path.

## Test plan

Unit tests (in-module, hermetic Context built literally — never `Context::resolve`,
which falls back to `$HOME/.config/opencode`):
- `apply_defaults` seeds all four slots on empty config/state; records them in state.json.
- `apply_defaults` skips slots whose `agent.<slot>.model` already exists or are tracked in state.
- `parse_models_output` extracts tokens, drops annotations/malformed lines, dedupes.
- `model_drift_findings` detects divergent values, state-only slots, untracked CE slots; ignores third-party slots.
- `import_config_assignments` imports missing/divergent entries, skips malformed, is idempotent.
