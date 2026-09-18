# Tasks: Pluggable Decision Engine & Jev Provider Foundation

## Estimated Scope: ~940 LOC across 5 atomic work units (~180–210 LOC per work unit)

- [x] **Work Unit 1: Domain Types, Question Primitives & Provider Trait** (est. ~180 LOC)
  - [x] Create `src/decisions/types.rs` with `DecisionQuestion` (`Boolean`, `Choice`, `Score`), `DecisionAnswer`, `DecisionRequest`, `DecisionResponse`, and `DecisionMode`.
  - [x] Create `src/decisions/mod.rs` with `DecisionProvider` trait and `DecisionEngine` dispatcher.
  - [x] Create `src/decisions/mock.rs` with `MockDecisionProvider` supporting customizable canned answers and failure modes.
  - [x] Add unit tests in `src/decisions/types.rs` and `src/decisions/mock.rs` validating serialization and offline evaluation.
  - [x] TDD Verification: `cargo test decisions::mock`

- [x] **Work Unit 2: Credential Resolution & Security Hygiene** (est. ~160 LOC)
  - [x] Create `src/decisions/auth.rs` implementing tiered resolution: `TYPESAFE_API_KEY` ➔ `JEV_API_KEY` ➔ `~/.config/ce-ai/credentials.toml`.
  - [x] Implement secure file write utility setting `0600` Unix permissions for `credentials.toml`.
  - [x] Implement key masking helper (e.g. `ts-****...****`) for terminal display.
  - [x] Add unit tests validating environment variable precedence, file fallback, and masking.
  - [x] TDD Verification: `cargo test decisions::auth`

- [x] **Work Unit 3: Budget Ceilings, Rate Limiting & Circuit Breaker** (est. ~200 LOC)
  - [x] Create `src/decisions/budget.rs` with `BudgetConfig`, `BudgetTracker`, and `CircuitBreaker`.
  - [x] Implement persistent monthly ledger tracking spend and request counts under `~/.config/ce-ai/decision_budget.json`.
  - [x] Implement graceful fallback: `BudgetTracker::can_execute()` returning `FallbackReason::BudgetExceeded` or `CircuitOpen` without erroring.
  - [x] Add unit tests validating monthly rollover, budget cutoff, and consecutive failure tripping (3 timeouts ➔ open circuit).
  - [x] TDD Verification: `cargo test decisions::budget`

- [x] **Work Unit 4: Jev (TypeSafe AI) HTTP Client Provider** (est. ~190 LOC)
  - [x] Create `src/decisions/jev.rs` implementing `DecisionProvider` for TypeSafe AI's Jev API.
  - [x] Map `DecisionQuestion` enums to Jev API JSON schema and parse responses with probability distributions.
  - [x] Wire timeout and connection pooling using `reqwest::blocking`.
  - [x] Implement `check_health()` endpoint ping for doctor integration.
  - [x] Add unit tests validating payload serialization and response parsing against sample Jev responses.
  - [x] TDD Verification: `cargo test decisions::jev`

- [x] **Work Unit 5: CLI Subcommand, Doctor Probe & Integration** (est. ~210 LOC)
  - [x] Create `src/commands/decisions.rs` implementing `ce-ai decisions [status|auth|setup|test]`.
  - [x] Register `decisions` subcommand in `src/main.rs`.
  - [x] Add `probe_decision_engine` in `src/commands/doctor.rs` with actionable setup recommendations.
  - [x] Add CLI integration tests in `tests/cli.rs` verifying `ce-ai decisions status` and doctor output.
  - [x] TDD Verification: `cargo test --test cli decisions` and `cargo run -- doctor`
