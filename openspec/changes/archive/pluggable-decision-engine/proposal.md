# Proposal: Pluggable Decision Engine & Jev Provider Foundation

## Problem Statement

`ce-ai` currently makes decisions through two distinct mechanisms:
1. **Deterministic logic in core Rust code**: Instantaneous, auditable, and immutable (e.g., FSM stage transitions, SHA256 integrity checks, path guards, git diff thresholds).
2. **Heavy LLM reasoning in AI harnesses**: Highly capable but characterized by noticeable latency (1.5s–10s+), high token costs, and non-deterministic behavior (e.g., full architecture brainstorming or large-scale refactoring).

A significant category of day-to-day decisions falls uncomfortably between these two extremes:
- Estimating whether a task requires a lightweight model or a high-reasoning model.
- Classifying task intent to suggest relevant skill categories before loading.
- Assessing whether an ambiguous CLI tool call (e.g. `rm`, shell script, network request) poses destructive risks.
- Semantically evaluating whether a task's Definition of Done (DoD) in an ODD brief or a stage checkpoint in Compound Engineering appears genuinely satisfied.

Attempting to solve these problems with brittle regex heuristics creates maintenance debt, while delegating every micro-classification to the primary LLM burns context and slows developer iterations.

Furthermore, integrating an external probabilistic decision provider like Jev (TypeSafe AI) introduces practical engineering challenges:
1. **Developer Experience Friction**: If configuring the provider requires complex manual JSON editing or risks credential leaks, adoption will stall.
2. **Unbounded Cost & Rate Limit Risks**: Without strict budget controls, cost ceilings, and circuit breakers, automated agent loops could generate unexpected API bills or lock up when quotas are exhausted.
3. **Fragility of External Dependencies**: If an external classification API fails, times out, or runs out of budget, the developer's workflow must **never** break.

This proposal introduces the **Pluggable Decision Engine (Phase 1)** into `ce-ai`: a provider-agnostic, lightweight **System One** advisory layer that sits between deterministic orchestration and heavy LLM reasoning. It features a clean `DecisionProvider` abstraction, first-class support for Jev (TypeSafe AI), seamless developer credential configuration, deterministic budget ceilings with circuit-breaker fallbacks, and developer onboarding suggestions via `ce-ai doctor`.

## In-Scope

1. **Provider-Agnostic Core Abstraction (`src/decisions/mod.rs`)**:
   - Define `DecisionProvider` trait (`evaluate(&self, req: DecisionRequest) -> Result<DecisionResponse, CeError>`).
   - Define typed question primitives decoupled from vendor types: `Boolean`, `Choice`, and `Score`.
   - Define typed answer primitives exposing confidence scores and option probabilities.
   - Implement `MockDecisionProvider` for 100% deterministic offline unit and integration testing.
2. **Jev (TypeSafe AI) Provider (`src/decisions/jev.rs`)**:
   - Implement HTTP client communicating with TypeSafe AI's Jev API using `reqwest` with timeouts and connection reuse.
   - Map `DecisionRequest` questions to Jev API payloads and parse structured responses.
3. **Frictionless & Secure API Key Management**:
   - Priority resolution: `TYPESAFE_API_KEY` (primary) ➔ `JEV_API_KEY` (fallback) ➔ optional local secured credential store `~/.config/ce-ai/credentials.toml` (0600 file permissions).
   - Invariant: Zero secret persistence in git-tracked files (`ce-ai.toml`, `state.json`) and automatic redaction in logs/CLI output.
   - CLI command `ce-ai decisions auth` for interactive entry or testing of API keys.
4. **Budget Ceilings, Rate Limiting & Graceful Circuit Breaker (`src/decisions/budget.rs`)**:
   - Configurable budget controls in `ce-ai.toml`: `max_monthly_usd`, `max_session_requests`, and `timeout_ms`.
   - Sliding-window tracking of accumulated estimated cost and request volume.
   - Consecutive failure circuit breaker (e.g. 3 consecutive timeouts/errors disables provider for the session).
   - **Hard Invariant (Graceful Fallback)**: If budget is exceeded, rate-limited, or circuit-broken, `ce-ai` logs an advisory warning and immediately falls back to deterministic defaults without failing the user command (exit code 0).
5. **Developer Configuration Presets & `doctor` Integration**:
   - CLI command `ce-ai decisions setup --preset recommended|shadow|local` providing ready-to-use configuration templates.
   - `ce-ai doctor` probe `probe_decision_engine` reporting provider availability, API key presence, latency, budget consumption, and actionable configuration suggestions.
6. **Execution Modes**:
   - `off`: Zero network calls or evaluations.
   - `shadow`: Evaluates decisions and records telemetry/latency in background without influencing execution.
   - `active`: Evaluations actively inform deterministic policies (e.g. model selection, risk warnings).

## Out-of-Scope

1. **Direct Workflow State Mutation**: The Decision Engine never modifies `state.json` or forces FSM stage transitions directly. It returns advisory evaluations consumed by deterministic policies.
2. **Replacing the Primary LLM / HarnessAdapter**: The Decision Engine classifies micro-decisions (System 1); code generation and interactive reasoning remain in the harness (OpenCode, Claude Code, etc.).
3. **Advanced Model / Skill / Risk Routing Logic**: These are defined in dedicated follow-up issues (#383, #384, #385, #386, #387). Phase 1 delivers the engine, Jev provider, budget controls, and setup experience.

## Risk Evaluation & Mitigation

- **Risk 1: Network Latency Hinders Developer Velocity.**
  - *Mitigation:* Strict configurable request timeout (default 1000ms). If timeout expires, immediate fallback to deterministic defaults.
- **Risk 2: Unexpected API Usage Bills.**
  - *Mitigation:* Hard monthly and per-session budget caps (`max_monthly_usd`, `max_session_requests`). When reached, the circuit breaker opens and execution gracefully degrades to deterministic rules.
- **Risk 3: API Key Leaks in Repositories.**
  - *Mitigation:* `ce-ai` forbids writing API keys into project-level configuration (`ce-ai.toml`). Keys are read from environment variables or global `~/.config/ce-ai/credentials.toml` protected by strict Unix `0600` file permissions.
- **Risk 4: Vendor Lock-in to Jev/TypeSafe AI.**
  - *Mitigation:* The `DecisionProvider` trait defines abstract domain types (`DecisionRequest`, `DecisionQuestion`). Jev is purely an implementation of this trait. Additional providers (`RuleProvider`, `LocalClassifierProvider`) can be added without modifying consumer code.

## Success Criteria

1. `cargo test` passes 100% using `MockDecisionProvider` without requiring external network connectivity or real credentials.
2. `ce-ai decisions status` displays provider status, model, latency, and budget metrics cleanly.
3. `ce-ai decisions auth` safely tests and configures API credentials.
4. `ce-ai doctor` detects missing, unconfigured, or degraded decision providers and suggests copy-pasteable configuration snippets.
5. In case of network errors, timeouts, or exceeded budgets, `ce-ai` completes commands successfully using deterministic fallbacks.
