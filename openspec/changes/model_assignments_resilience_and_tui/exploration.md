# Exploration: Model Assignments Resilience and TUI Editor

## Technical Investigation

### 1. State & Harness Configuration Interplay
- **`src/state/state.rs`**: `State` holds `pub model_assignments: HashMap<String, ModelAssignment>`. `ModelAssignment` stores `model: String` and `updated_at: String`.
- **`src/opencode/config.rs`**: `apply_model_assignment(&mut json, slot, model_name)` writes `agent.<slot> = {"model": model_name, "variant": ""}`.
- **Root Cause of Issue #111**: When `state.json` is missing or reset during re-installation, `state.json` loses `model_assignments`. However, `opencode.json` retains `agent.<slot>`. Consequently, `ce-ai models list` queries `state.json` and outputs `(none)`, creating silent drift.

### 2. `ce-ai doctor` Health Probes
- **Current Behavior**: `doctor.rs` checks binary versions, file diffs, git hooks, and branch protection, but does not read `opencode.json`'s `agent` section to check for unregistered model assignments.
- **Proposed Probe**: Read active harness configs (`opencode.json`). Compare `agent.<slot>.model` entries against `state.model_assignments`. If entries exist in `opencode.json` but not in `state.json` (or vice-versa), output a `Finding::Warning` or `Finding::Info`.

### 3. `ce-ai sync` Reconciliation Path
- **Current Behavior**: `sync.rs` restores missing managed files and updates `manifest.json`.
- **Proposed Reconciliation**: During `ce-ai sync`, parse active harness configs. Any `agent.<slot>` found in `opencode.json` that is unrecorded in `state.json` is populated into `state.json`. Conversely, any assignment in `state.json` missing from `opencode.json` is synced to `opencode.json`.

### 4. TUI Models Tab Interactivity
- **Current Behavior**: `src/tui.rs` renders read-only tables of model assignments and profiles.
- **Proposed Enhancement**:
  - Track `selected_slot: usize` and `editing_model: bool` in `TuiApp`.
  - Handle keyboard navigation (`Up`/`Down`, `j`/`k`).
  - Pressing `e` or `Enter` opens an inline model string editor modal or input field.
  - Pressing `Enter` in edit mode invokes `commands::models::set_model()` and saves atomically.
