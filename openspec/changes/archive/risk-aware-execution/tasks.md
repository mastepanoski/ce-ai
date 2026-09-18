# Tasks: Risk-Aware Tool Execution (Intelligent Permission & Risk Engine)

## Estimated Scope: ~700 LOC across 4 atomic work units (~160–190 LOC per work unit)

- [x] **Work Unit 1: Data Structures, Configuration Schema & Deterministic Rule Engine** (est. ~170 LOC)
  - [x] Define `ExecutionPolicy`, `RiskFallbackPolicy`, `RiskThresholds`, `RiskConfig` in `src/decisions/risk.rs`.
  - [x] Extend `DecisionsConfig` in `src/state/state.rs` with `pub risk: RiskConfig`.
  - [x] Implement deterministic security rule pre-filter (`check_deterministic_denial`) and safe command check (`is_safe_read_only`).
  - [x] Implement credential and sensitive argument sanitizer (`redact_sensitive_content`).
  - [x] Add unit tests verifying deterministic denials, safe passes, redaction, and `Eq` trait compliance.
  - [x] TDD Verification: `cargo test decisions::risk::tests::deterministic`

- [x] **Work Unit 2: Probabilistic Risk Evaluator & Composite Scoring** (est. ~190 LOC)
  - [x] Implement `RiskEvaluator` in `src/decisions/risk.rs` with 6 semantic risk questions (`destructive`, `credential_sensitive`, `external_side_effect`, `privilege_escalation`, `irreversible`, `scope_exceeds_task`).
  - [x] Implement composite risk calculation and threshold mapping (`confirmation_threshold_pct`, `deny_threshold_pct`).
  - [x] Implement conservative fail-closed fallback logic (`RequireConfirmation` / `Deny`).
  - [x] Implement audit event logger recording sanitized evaluations to `risk-events.jsonl`.
  - [x] Add unit tests in `src/decisions/tests/risk_tests.rs` with `MockDecisionProvider` simulating allow, confirm, deny, and timeout fallback scenarios.
  - [x] TDD Verification: `cargo test decisions::risk::tests::evaluation`

- [x] **Work Unit 3: CLI Subcommand `ce-ai decisions check-risk` & Setup Presets** (est. ~170 LOC)
  - [x] Add `CheckRisk` action to `src/commands/decisions.rs` accepting tool, command, optional task prompt, `--json`, and `--verbose`.
  - [x] Update `Action::Setup` presets (`recommended`, `shadow`, `local`) to configure `RiskConfig`.
  - [x] Format human-readable, `--json`, and `--verbose` score outputs.
  - [x] Add unit tests verifying argument parsing, formatting, and JSON serialization.
  - [x] TDD Verification: `cargo test commands::decisions::tests`

- [x] **Work Unit 4: Doctor Probe Integration, Full Matrix Tests & Quality Gates** (est. ~170 LOC)
  - [x] Update `probe_decision_engine_health` in `src/commands/doctor.rs` to report risk evaluation engine status and active fallback policy.
  - [x] Add end-to-end CLI integration test in `tests/cli.rs` (`test_cli_decisions_check_risk_deterministic_and_probabilistic`).
  - [x] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`).
  - [x] TDD Verification: `cargo test --test cli check_risk`
