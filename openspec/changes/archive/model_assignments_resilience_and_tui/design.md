# Design: Model Assignments Resilience and TUI Editor

## Architectural Changes & Data Schemas

```
                       ┌───────────────────────────────┐
                       │   ce-ai doctor / ce-ai sync   │
                       └───────────────┬───────────────┘
                                       │
                      Bi-directional Drift Detection
                                       │
            ┌──────────────────────────┴──────────────────────────┐
            ▼                                                     ▼
┌───────────────────────┐                             ┌───────────────────────┐
│   ~/.ce-ai/state.json │ ◄────── Sync & Repair ────► │    opencode.json      │
│  (model_assignments)  │                             │  (agent.<slot>.model) │
└───────────────────────┘                             └───────────────────────┘
```

### 1. Default Model Assignments Map
Define standard default model assignments for Compound Engineering workflow slots:
- `ce-ai`: `anthropic/claude-3-7-sonnet` (Orchestrator)
- `ce-brainstorm`: `anthropic/claude-3-7-sonnet`
- `ce-plan`: `anthropic/claude-3-7-sonnet`
- `ce-work`: `anthropic/claude-3-7-sonnet`
- `ce-code-review`: `anthropic/claude-3-7-sonnet`
- `ce-doc-review`: `anthropic/claude-3-7-sonnet`

### 2. `ce-ai doctor` Health Probe Design
Function `check_model_assignments_health(state: &State, home: &Path) -> Vec<Finding>`:
- Parse `~/.config/opencode/opencode.json`.
- Extract keys under `"agent"`.
- If an `agent.<slot>` key exists with a `"model"` string, check if `state.model_assignments` contains `<slot>`.
- If missing from `state.model_assignments`, emit `Finding::Warning`:
  `"Model assignment drift: agent.<slot> configured in opencode.json but unrecorded in state.json. Run 'ce-ai sync' to reconcile."`

### 3. `ce-ai sync` Reconciliation Logic
In `src/commands/sync.rs`:
- Read `opencode.json`.
- For each `agent.<slot>` entry in `opencode.json`, if `<slot>` is missing in `state.model_assignments`, insert `ModelAssignment { model, updated_at }` into `state.model_assignments`.
- For each `<slot>` in `state.model_assignments`, ensure `opencode.json` contains `agent.<slot>`.
- Write both files atomically via `write_atomic`.

### 4. Interactive TUI Model Assignment Editor Design
In `src/tui.rs`:
- Add fields to `TuiApp`:
  - `model_slots: Vec<(String, String)>` (slot name, model name)
  - `selected_slot_index: usize`
  - `is_editing_model: bool`
  - `edit_buffer: String`
- Keys:
  - `j` / `Down`: Move selection down
  - `k` / `Up`: Move selection up
  - `e` / `Enter`: Toggle edit mode for selected slot
  - `Esc`: Cancel editing
  - `Enter` (when editing): Commit model assignment via `commands::models::set_model()` and refresh TUI view.
