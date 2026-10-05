# Tasks: Phase 3 — Advisory Workflow FSM & OpenSpec Decoupling (v1.77.0)

## Work Units & Changed-Line Estimates

- [ ] **Unit 1: Implement `src/observation/` Data Structs & Engine** (~200 LOC)
  - [ ] Define `ObservableWorkflowState`, `PlanObservation`, `VerificationObservation`, `HandoffObservation`, `KnowledgeObservation`, `OpenSpecObservation` in `src/observation/state.rs`.
  - [ ] Implement `ObservableWorkflowState::observe(repo_root: &Path, config_dir: &Path) -> Result<Self, CeError>`.
  - [ ] Implement markdown plan parsing and checkbox counting (`- [x]` vs `- [ ]`) respecting `CeDocsConfig::plans_dir`.
  - [ ] Implement git branch and uncommitted change detection.
  - [ ] Implement optional OpenSpec inspection returning `None` when inactive.
  - [ ] Export `src/observation` in `src/lib.rs` / `src/main.rs`.

- [ ] **Unit 2: Add Observation Subsystem Unit Tests** (~180 LOC)
  - [ ] Create `src/observation/tests/` module with tests covering:
    - Empty repository without plans or OpenSpec (returns default observation, `openspec: None`).
    - Repository with active plan and checkbox counts.
    - Repository with uncommitted files (`active_work: true`).
    - Repository with optional OpenSpec change folder detected.
    - Verification receipt stamped state.

- [ ] **Unit 3: Gate Policy & Default Mode Refinement** (~180 LOC)
  - [ ] Update `resolve_gate_mode` in `src/commands/gate.rs` to default to `GateMode::Observe` when unspecified.
  - [ ] Update `evaluate_gate_policy` to never return `GateDecision::Blocked` in `GateMode::Observe`.
  - [ ] Ensure absence of `openspec/changes/` on non-OpenSpec tasks produces non-blocking `Pass` or advisory `WouldBlock`.
  - [ ] Update gate unit tests in `src/commands/tests/gate.rs` to verify non-blocking write behavior under default settings.

- [ ] **Unit 4: Workflow Status Diagnostic Integration** (~190 LOC)
  - [ ] Update `status_lines_with_mode` in `src/commands/workflow.rs` to invoke `ObservableWorkflowState::observe`.
  - [ ] Render the `Advisory Workflow Capabilities Matrix` in stdout formatting.
  - [ ] Update `Action::Status { json: true, .. }` to serialize `observable_state` in the JSON response payload.
  - [ ] Update workflow CLI integration tests in `tests/cli.rs`.

- [ ] **Unit 5: Operating Directives & Domain Vocabulary Accretion** (~120 LOC)
  - [ ] Update `AGENTS.md` Invariant 6 to formalize "No Semantic Authority & Optional OpenSpec".
  - [ ] Monotonically accrete new architectural concepts in `CONCEPTS.md`:
    - `Observable Workflow Capabilities Matrix`
    - `No Semantic Authority Principle`
    - `Decoupled OpenSpec Integration`
    - `Advisory Write Telemetry`
  - [ ] Verify concept integrity with `cargo run -- doc lint --strict`.
  - [ ] Document solution in `docs/solutions/architecture/` per Compound Engineering standards.
  - [ ] Bump version to `1.77.0` in `Cargo.toml` and update `CHANGELOG.md`.

- [ ] **Unit 6: Verification & Quality Gates** (0 LOC)
  - [ ] Verify formatting: `cargo fmt --check`.
  - [ ] Verify linter: `cargo clippy --all-targets --all-features -- -D warnings`.
  - [ ] Verify test suite: `cargo test`.
  - [ ] Verify E2E gate: `make e2e`.
