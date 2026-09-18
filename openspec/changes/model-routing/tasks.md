# Tasks: Adaptive Model Route Selection (System One Advisory Layer)

## Estimated Scope: ~650 LOC across 4 atomic work units (~150–190 LOC per work unit)

- [ ] **Work Unit 1: State Schema & Domain Types** (est. ~150 LOC)
  - [ ] Define `ModelClass` (`Fast`, `Standard`, `Reasoning`) with string parsing and display in `src/decisions/routing.rs`.
  - [ ] Define `ModelClassCatalog` (`fast`, `standard`, `reasoning`) with fallback getters.
  - [ ] Define `RoutingThresholds` (`reasoning_threshold_pct`, `standard_threshold_pct`) with safe defaults.
  - [ ] Extend `DecisionsConfig` in `src/state/state.rs` with `pub routing: ModelRoutingConfig`.
  - [ ] Add unit tests in `src/decisions/routing.rs` and `src/state/state.rs` verifying serialization, defaults, and `Eq` trait compliance.
  - [ ] TDD Verification: `cargo test decisions::routing::tests::types`

- [ ] **Work Unit 2: Core Routing Policy Engine & Fallback Chain** (est. ~190 LOC)
  - [ ] Implement `ModelRouter` in `src/decisions/routing.rs`.
  - [ ] Implement question formulation (`complexity`, `needs_reasoning`, `needs_large_context`, `risk`).
  - [ ] Implement deterministic mapping from `DecisionResponse` answers to `ModelClass`.
  - [ ] Implement capability fallback chain (`reasoning` ➔ `standard` ➔ `fast` ➔ default).
  - [ ] Enforce override precedence: explicit override > static slot assignment > adaptive routing > default.
  - [ ] Add unit tests in `src/decisions/routing.rs` simulating all decision permutations, timeouts, and missing catalog slots with `MockDecisionProvider`.
  - [ ] TDD Verification: `cargo test decisions::routing::tests::policy`

- [ ] **Work Unit 3: CLI Subcommand `ce-ai models route`** (est. ~160 LOC)
  - [ ] Add `Route(RouteArgs)` to `ModelsCommand` in `src/commands/models.rs`.
  - [ ] Support `--json` and `--verbose` output formatting.
  - [ ] Alias `Route` in `ce-ai decisions route` to route via `handle_route` in `src/commands/decisions.rs`.
  - [ ] Add unit tests verifying CLI argument parsing and error handling in `src/commands/tests/models.rs`.
  - [ ] TDD Verification: `cargo test commands::tests::models`

- [ ] **Work Unit 4: Setup Presets, Doctor Probe Integration & CLI Tests** (est. ~150 LOC)
  - [ ] Update `ce-ai decisions setup --preset recommended` to populate default model classes in `routing.models` and enable routing.
  - [ ] Update `probe_decision_engine` in `src/commands/doctor.rs` to display adaptive routing status and configured model catalog.
  - [ ] Add end-to-end CLI integration tests in `tests/cli.rs` testing `ce-ai models route` with `--json` and fallback behaviors.
  - [ ] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`).
  - [ ] TDD Verification: `cargo test --test cli models_route`
