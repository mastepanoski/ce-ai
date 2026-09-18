# Specification: Pluggable Decision Engine & Jev Provider Foundation

## Requirements

### R1: Provider-Agnostic Decision Interface
- **WHEN** a component invokes the Decision Engine with a `DecisionRequest`
- **THEN** it MUST interact strictly via the `DecisionProvider` trait using typed domain primitives (`Boolean`, `Choice`, `Score`).
- **WHEN** no external decision provider is configured or decisions are disabled (`enabled = false`)
- **THEN** the engine MUST return a deterministic default response or report `Disabled` without erroring.

### R2: Jev (TypeSafe AI) Provider Integration
- **WHEN** `provider = "jev"` is configured in `[decisions]`
- **THEN** the engine MUST format requests according to TypeSafe AI's Jev API schema, submit via HTTPS with the configured timeout, and deserialize the response into domain `DecisionAnswer` instances with confidence metrics.
- **WHEN** the Jev API returns an HTTP error (4xx, 5xx) or times out
- **THEN** the engine MUST record the failure in the circuit breaker and fall back gracefully to deterministic defaults.

### R3: Secure & Frictionless Credential Resolution
- **WHEN** resolving the API key for Jev
- **THEN** `ce-ai` MUST inspect `TYPESAFE_API_KEY` first, followed by `JEV_API_KEY`, followed by `~/.config/ce-ai/credentials.toml`.
- **WHEN** persisting credentials via `ce-ai decisions auth`
- **THEN** it MUST write exclusively to the user's private config file (`~/.config/ce-ai/credentials.toml`) with strict `0600` permissions (on Unix).
- **WHEN** writing or displaying configuration
- **THEN** `ce-ai` MUST NEVER serialize API keys into git-tracked files (`ce-ai.toml`, `state.json`) or print unmasked keys in CLI output or logs.

### R4: Budget Ceilings, Rate Limits & Circuit Breaker
- **WHEN** a decision request is initiated
- **THEN** the `BudgetTracker` MUST check if accumulated monthly spend exceeds `max_monthly_usd` or if session requests exceed `max_session_requests`.
- **WHEN** the budget ceiling is exceeded
- **THEN** the Decision Engine MUST emit an advisory warning (`⚠ Decision Engine budget exceeded — falling back to deterministic defaults`) and bypass provider invocation with exit code 0.
- **WHEN** `max_consecutive_failures` (default: 3) consecutive errors or timeouts occur
- **THEN** the circuit breaker MUST trip to `Open`, bypassing further network calls for the duration of the cool-off period.

### R5: Developer Setup & Onboarding Ergonomics
- **WHEN** a developer runs `ce-ai decisions setup --preset recommended`
- **THEN** `ce-ai` MUST populate `ce-ai.toml` with safe defaults (`active` mode, `max_monthly_usd = 5.0`, `timeout_ms = 1000`).
- **WHEN** a developer runs `ce-ai doctor` without a configured decision provider
- **THEN** `ce-ai doctor` MUST emit an informative notice explaining the benefits of the Decision Engine along with ready-to-run setup commands (`ce-ai decisions setup` or environment variable instructions).
- **WHEN** a developer runs `ce-ai doctor` with an active provider
- **THEN** `ce-ai doctor` MUST probe provider latency, verify API key validity, and report monthly budget utilization.

### R6: Execution Modes (Active, Shadow, Off)
- **WHEN** `mode = "shadow"` is configured
- **THEN** the Decision Engine MUST evaluate the request and record latency/confidence telemetry, but execution policies MUST ignore the probabilistic outcome and rely entirely on deterministic behavior.
- **WHEN** `mode = "off"` is configured
- **THEN** zero network calls or provider evaluations MUST be made.

---

## Acceptance Criteria

1. **Deterministic Offline Tests**: Unit and integration tests in `cargo test` pass 100% without network access using `MockDecisionProvider`.
2. **Safe Credential Handling**: `credentials.toml` creation enforces `0600` permissions on Unix platforms; API keys are never leaked to stdout or git-tracked files.
3. **Budget Guard Invariant**: Synthetically exceeding the budget ceiling immediately diverts to deterministic fallback and logs an advisory warning without failing the command.
4. **Circuit Breaker Invariant**: Three consecutive mock timeouts trigger circuit open, avoiding repeated network stalls.
5. **Doctor Diagnostic Parity**: `ce-ai doctor` cleanly identifies all provider states (`Unconfigured`, `Healthy`, `Degraded`, `BudgetExceeded`).
