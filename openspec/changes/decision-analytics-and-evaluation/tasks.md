# Tasks: Decision Analytics & Evaluation

## Estimated Scope: ~740 LOC across 4 atomic work units (~180–190 LOC per work unit)

- [x] **Work Unit 1: Data Structures, Configuration Schema & Event Ledger** (est. ~180 LOC)
  - [x] Define `DecisionType`, `DecisionEvent`, `DecisionAnalyticsConfig`, `LatencyStats`, `ConfidenceStats`, `DecisionStats`, `RoutingComparison`, and `CostValue` in `src/decisions/analytics.rs`.
  - [x] Extend `DecisionsConfig` in `src/state/state.rs` with `pub analytics: DecisionAnalyticsConfig`.
  - [x] Implement `decision_ledger_path`, `log_decision_event`, and `read_decision_events` in `src/decisions/analytics.rs` reusing `src/capture/ledger.rs` atomic append conventions.
  - [x] Re-export analytics primitives in `src/decisions/mod.rs`.
  - [x] Add unit tests for serialization, deserialization, ledger atomic writes, and malformed line tolerance.
  - [x] TDD Verification: `cargo test decisions::analytics::tests::ledger`

- [x] **Work Unit 2: Analytics Engine, Metrics Aggregation & Cost Model** (est. ~190 LOC)
  - [x] Implement `DecisionAnalytics` in `src/decisions/analytics.rs` with filtering by `workflow_id` and `decision_type`.
  - [x] Implement latency percentile calculations (min, median, p95, mean, max) and confidence distributions.
  - [x] Implement `compute_comparison` comparing static vs adaptive model routing, calculating observed costs from `UsageRecord`s or benchmark estimated costs.
  - [x] Ensure clear separation between `[observed]` and `[estimated]` cost values.
  - [x] Add unit tests for aggregation math, fallback rates, and cost calculations under various data conditions (empty, single, multiple).
  - [x] TDD Verification: `cargo test decisions::analytics::tests::aggregation`

- [x] **Work Unit 3: Evaluator Instrumentation & Shadow Mode Telemetry** (est. ~180 LOC)
  - [x] Add telemetry logging hook to `ModelRouter::route` for capturing model routing events and counterfactual shadow events.
  - [x] Add telemetry logging hooks for `SkillRouter`, `RiskEvaluator`, and `ReadinessEvaluator`.
  - [x] Enforce strict privacy sanitization: redact sensitive details, prompts, and raw commands before logging.
  - [x] Validate that shadow mode logs events with `shadow_mode: true` while preserving authoritative default behaviors.
  - [x] Add unit tests verifying event emission across all four decision evaluators and in shadow mode.
  - [x] TDD Verification: `cargo test decisions::analytics::tests::instrumentation`

- [x] **Work Unit 4: CLI Subcommands (`stats` & `compare`), Doctor Probe, & Verification** (est. ~190 LOC)
  - [x] Add `Stats` and `Compare` subcommands to `Action` enum in `src/commands/decisions.rs`.
  - [x] Implement `handle_stats` and `handle_compare` supporting human-readable formatted tables and machine-readable `--json`.
  - [x] Update `ce-ai doctor` in `src/commands/doctor.rs` with a decision analytics health check reporting recorded event volume and fallback rates.
  - [x] Add end-to-end CLI integration tests in `tests/cli.rs` (`test_cli_decisions_stats_and_compare`).
  - [x] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`).
  - [x] TDD Verification: `cargo test --test cli decisions_stats_and_compare`
