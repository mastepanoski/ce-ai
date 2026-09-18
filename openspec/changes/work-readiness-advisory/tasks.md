# Tasks: Work Readiness & Verification Advisory

## Estimated Scope: ~700 LOC across 4 atomic work units (~170–180 LOC per work unit)

- [x] **Work Unit 1: Data Structures, Configuration Schema & Core Primitives** (est. ~170 LOC)
  - [x] Define `ReadinessStatus`, `ReadinessThresholds`, `ReadinessConfig`, `ReadinessDimensionScore`, `ReadinessEvaluationResult` in `src/decisions/readiness.rs`.
  - [x] Extend `DecisionsConfig` in `src/state/state.rs` with `pub readiness: ReadinessConfig`.
  - [x] Define standardized dimension question constants for ODD (`ODD_READINESS_DIMENSIONS`) and CE (`CE_READINESS_DIMENSIONS`).
  - [x] Export `pub mod readiness;` and re-exports in `src/decisions/mod.rs`.
  - [x] Add unit tests for configuration defaults, serde roundtrips, and `Eq` trait compliance.
  - [x] TDD Verification: `cargo test decisions::readiness::tests::config`

- [x] **Work Unit 2: Semantic Readiness Evaluator & Composite Scoring** (est. ~180 LOC)
  - [x] Implement `ReadinessEvaluator` in `src/decisions/readiness.rs` with `evaluate_odd` and `evaluate_stage`.
  - [x] Implement composite scoring calculation and threshold mapping (`ready_pct: 80`, `warning_pct: 60`).
  - [x] Implement graduation advisory trigger when `graduation_recommended` is elevated.
  - [x] Implement fail-safe graceful degradation when decision provider is offline or times out.
  - [x] Add unit tests in `src/decisions/tests/readiness_tests.rs` with `MockDecisionProvider` testing Ready, Warning, NotReady, and fallback scenarios.
  - [x] TDD Verification: `cargo test decisions::readiness::tests::evaluator`

- [x] **Work Unit 3: CLI Subcommand `ce-ai decisions check-readiness` & Setup Presets** (est. ~170 LOC)
  - [x] Add `CheckReadiness` action to `src/commands/decisions.rs` accepting optional feature, task, stage, `--json`, and `--verbose`.
  - [x] Implement `handle_check_readiness` in `src/commands/decisions.rs` formatting human-readable and structured outputs.
  - [x] Update `Action::Setup` presets (`recommended`, `shadow`, `local`) to configure `ReadinessConfig`.
  - [x] Add unit tests in `src/commands/decisions.rs` testing CLI handler and preset provisioning.
  - [x] TDD Verification: `cargo test commands::decisions::tests::readiness`

- [x] **Work Unit 4: Workflow Status Integration, Doctor Probe & Full Quality Gates** (est. ~180 LOC)
  - [x] Update `probe_decision_engine_health` in `src/commands/doctor.rs` to report readiness engine status and thresholds.
  - [x] Integrate readiness advisory badge into `ce-ai workflow status` when readiness evaluation is enabled.
  - [x] Add end-to-end CLI integration test in `tests/cli.rs` (`test_cli_decisions_check_readiness`).
  - [x] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`).
  - [x] TDD Verification: `cargo test --test cli check_readiness`
