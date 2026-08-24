> STATUS (v1.20.1): models set/list/profile live in src/commands/models.rs. Residual open boxes below were not re-audited item-by-item.

# Tasks: Model Assignments Resilience and TUI Editor

## Implementation Checklist

- [ ] **Task 1: Default Model Assignments in Install**
  - Define `DEFAULT_MODEL_ASSIGNMENTS` in `src/state/state.rs` or `src/commands/models.rs`.
  - Update `src/commands/install.rs` to apply default assignments on fresh install when slots are unconfigured.

- [ ] **Task 2: Doctor Health Probe for Model Assignment Drift**
  - Implement `check_model_assignments_health` in `src/commands/doctor.rs`.
  - Parse `opencode.json` `agent` section and compare against `state.model_assignments`.
  - Emit diagnostic warnings when drift is detected.

- [ ] **Task 3: Sync Reconciliation for Model Assignments**
  - Extend `src/commands/sync.rs` to parse harness configs and update `state.model_assignments` and `opencode.json`.
  - Ensure `crate::state::write_atomic` is used for all file mutations.

- [ ] **Task 4: Interactive TUI Model Assignment Editor**
  - Update `src/tui.rs` `TuiApp` state to support slot selection and inline editing.
  - Implement key event handlers (`j`/`k`, `Up`/`Down`, `e`, `Enter`, `Esc`).
  - Wire editing action to `commands::models::set_model()`.

- [ ] **Task 5: Integration Tests & DoD Verification**
  - Add integration tests in `tests/cli.rs` testing `doctor` drift detection and `sync` reconciliation.
  - Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e`.
