# Tasks: Adaptive Model Route Selection (System One Advisory Layer)

## Estimated Scope: ~650 LOC across 4 atomic work units (~150–190 LOC per work unit)

- [x] **Work Unit 1: State Schema & Domain Types** (est. ~150 LOC)
  - [x] Define `ModelClass` (`Fast`, `Standard`, `Reasoning`) with string parsing and display in `src/decisions/routing.rs`.
  - [x] Define `ModelClassCatalog` (`fast`, `standard`, `reasoning`) with fallback getters.
  - [x] Define `RoutingThresholds` (`reasoning_threshold_pct`, `standard_threshold_pct`) with safe defaults.
  - [x] Extend `DecisionsConfig` in `src/state/state.rs` with `pub routing: ModelRoutingConfig`.
  - [x] Add unit tests in `src/decisions/routing.rs` and `src/state/state.rs` verifying serialization, defaults, and `Eq` trait compliance.
  - [x] TDD Verification: `cargo test decisions::routing::tests::types`

- [x] **Work Unit 2: Core Routing Policy Engine & Fallback Chain** (est. ~190 LOC)
  - [x] Implement `ModelRouter` in `src/decisions/routing.rs`.
  - [x] Implement question formulation (`complexity`, `needs_reasoning`, `needs_large_context`, `risk`).
  - [x] Implement deterministic mapping from `DecisionResponse` answers to `ModelClass`.
  - [x] Implement capability fallback chain (`reasoning` ➔ `standard` ➔ `fast` ➔ default).
  - [x] Enforce override precedence: explicit override > static slot assignment > adaptive routing > default.
  - [x] Add unit tests in `src/decisions/routing.rs` simulating all decision permutations, timeouts, and missing catalog slots with `MockDecisionProvider`.
  - [x] TDD Verification: `cargo test decisions::routing::tests::policy`

- [x] **Work Unit 3: CLI Subcommand `ce-ai models route`** (est. ~160 LOC)
  - [x] Add `Route(RouteArgs)` to `ModelsCommand` in `src/commands/models.rs`.
  - [x] Support `--json` and `--verbose` output formatting.
  - [x] Alias `Route` in `ce-ai decisions route` to route via `handle_route` in `src/commands/decisions.rs`.
  - [x] Add unit tests verifying CLI argument parsing and error handling in `src/commands/tests/models.rs`.
  - [x] TDD Verification: `cargo test commands::tests::models`

- [x] **Work Unit 4: Setup Presets, Doctor Probe Integration & CLI Tests** (est. ~150 LOC)
  - [x] Update `ce-ai decisions setup --preset recommended` to populate default model classes in `routing.models` and enable routing.
  - [x] Update `probe_decision_engine` in `src/commands/doctor.rs` to display adaptive routing status and configured model catalog.
  - [x] Add end-to-end CLI integration tests in `tests/cli.rs` testing `ce-ai models route` with `--json` and fallback behaviors.
  - [x] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`).
  - [x] TDD Verification: `cargo test --test cli models_route`
