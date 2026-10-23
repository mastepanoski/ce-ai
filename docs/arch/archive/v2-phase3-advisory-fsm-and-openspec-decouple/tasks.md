# Tasks: Phase 3 — Advisory Workflow FSM & OpenSpec Decoupling (v1.77.0)

## Work Units & Changed-Line Estimates

- [x] **Unit 1: Implement `src/observation/` Data Structs & Engine** (~200 LOC)
  - [x] Define `ObservableWorkflowState`, `PlanObservation`, `VerificationObservation`, `HandoffObservation`, `KnowledgeObservation`, `OpenSpecObservation` in `src/observation/state.rs`.
  - [x] Implement `ObservableWorkflowState::observe(repo_root: &Path, config_dir: &Path) -> Result<Self, CeError>`.
  - [x] Implement markdown plan parsing and checkbox counting (`- [x]` vs `- [ ]`) respecting `CeDocsConfig::plans_dir`.
  - [x] Implement git branch and uncommitted change detection.
  - [x] Implement optional OpenSpec inspection returning `None` when inactive.
  - [x] Export `src/observation` in `src/lib.rs` / `src/main.rs`.

- [x] **Unit 2: Add Observation Subsystem Unit Tests** (~180 LOC)
  - [x] Create `src/observation/tests/` module with tests covering:
    - [x] Empty repository without plans or OpenSpec (returns default observation, `openspec: None`).
    - [x] Repository with active plan and checkbox counts.
    - [x] Repository with uncommitted files (`active_work: true`).
    - [x] Repository with optional OpenSpec change folder detected.
    - [x] Verification receipt stamped state.

- [x] **Unit 3: Gate Policy & Default Mode Refinement** (~180 LOC)
  - [x] Update `resolve_gate_mode` in `src/commands/gate.rs` to default to `GateMode::Observe` when unspecified.
  - [x] Update `evaluate_gate_policy` to never return `GateDecision::Blocked` in `GateMode::Observe`.
  - [x] Ensure absence of `openspec/changes/` on non-OpenSpec tasks produces non-blocking `Pass` or advisory `WouldBlock`.
  - [x] Update gate unit tests in `src/commands/tests/gate.rs` to verify non-blocking write behavior under default settings.

- [x] **Unit 4: Workflow Status Diagnostic Integration** (~190 LOC)
  - [x] Update `status_lines_with_mode` in `src/commands/workflow.rs` to invoke `ObservableWorkflowState::observe`.
  - [x] Render the `Advisory Workflow Capabilities Matrix` in stdout formatting.
  - [x] Update `Action::Status { json: true, .. }` to serialize `observable_state` in the JSON response payload.
  - [x] Update workflow CLI integration tests in `tests/cli.rs`.

- [x] **Unit 5: Operating Directives & Domain Vocabulary Accretion** (~120 LOC)
  - [x] Update `AGENTS.md` Invariant 6 to formalize "No Semantic Authority & Optional OpenSpec".
  - [x] Monotonically accrete new architectural concepts in `CONCEPTS.md`:
    - [x] `Observable Workflow Capabilities Matrix`
    - [x] `No Semantic Authority Principle`
    - [x] `Decoupled OpenSpec Integration`
    - [x] `Advisory Write Telemetry`
  - [x] Verify concept integrity with `cargo run -- doc lint --strict`.
  - [x] Document solution in `docs/solutions/architecture/` per Compound Engineering standards.
  - [x] Bump version to `1.77.0` in `Cargo.toml` and update `CHANGELOG.md`.

- [x] **Unit 6: Verification & Quality Gates** (0 LOC)
  - [x] Verify formatting: `cargo fmt --check`.
  - [x] Verify linter: `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Verify test suite: `cargo test`.
  - [x] Verify E2E gate: `make e2e`.
