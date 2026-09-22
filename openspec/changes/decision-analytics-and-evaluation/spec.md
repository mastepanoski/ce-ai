# Specification: Decision Analytics & Evaluation

## Formal Requirements

### Requirement 1: Telemetry Event Logging
- **WHEN** any Decision Engine evaluation occurs (`model_routing`, `skill_routing`, `risk_classification`, or `stage_readiness`) and `analytics.enabled` is `true`,
- **THEN** the system MUST append a structured `DecisionEvent` record to `.ce-ai/usage/decisions.jsonl`.
- **AND** the event MUST contain a unique ID, timestamp, decision type, provider, model, latency, confidence, outcome, and fallback indicator.
- **AND** the write operation MUST use atomic appending without blocking or corrupting concurrent accesses.

### Requirement 2: Privacy and State Redaction
- **WHEN** recording telemetry for any decision evaluation,
- **THEN** raw source code, prompts, credentials, full conversations, and raw shell/tool arguments MUST NOT be stored.
- **AND** only sanitized metadata (such as task complexity, risk dimension rating, tool name, or truncated summary hash) MAY be persisted.

### Requirement 3: Counterfactual & Shadow Evaluation
- **WHEN** `decisions.mode` is set to `shadow`,
- **THEN** the Decision Engine MUST evaluate the request and log a `DecisionEvent` with `shadow_mode: true`.
- **AND** deterministic behavior MUST remain authoritative and uninfluenced by the evaluated decision.
- **AND** counterfactual comparison MUST be able to compare the shadow decision against the authoritative execution.

### Requirement 4: Workflow Correlation
- **WHEN** a decision evaluation occurs within an active workflow (or `--workflow <id>` context is provided),
- **THEN** the `workflow_id` and `stage` MUST be captured in the `DecisionEvent`.
- **AND** queries to `ce-ai decisions stats --workflow <id>` and `ce-ai decisions compare --workflow <id>` MUST filter records strictly to that workflow.

### Requirement 5: Metrics Aggregation & CLI Output
- **WHEN** a user executes `ce-ai decisions stats`,
- **THEN** the system MUST output total runs, fallback percentage, median/p95 latency, average confidence, and type distribution.
- **WHEN** the `--json` flag is provided to `ce-ai decisions stats` or `ce-ai decisions compare`,
- **THEN** the system MUST output valid, structured JSON.

### Requirement 6: Cost Comparison & Ground-Truth Distinction
- **WHEN** a user executes `ce-ai decisions compare`,
- **THEN** the system MUST report model routing distribution (`fast %`, `standard %`, `reasoning %`), fallback rate, and median decision latency.
- **AND** the system MUST report static vs adaptive routing cost.
- **AND** every reported cost value MUST explicitly declare whether it is `[observed]` (derived from token usage in the usage ledger) or `[estimated]` (derived from benchmark complexity pricing).

### Requirement 7: Graceful Degradation & Non-Disruption
- **WHEN** the decision events ledger file is missing or contains malformed lines,
- **THEN** the system MUST skip malformed lines with a warning, report valid records, and exit with code 0.
- **WHEN** writing a decision event fails (e.g. read-only filesystem or disk full),
- **THEN** the core decision operation MUST NOT fail and the command MUST complete successfully.

### Requirement 8: Provider Agnosticism
- **WHEN** any configured provider (`jev`, `mock`, or future providers) evaluates decisions,
- **THEN** telemetry events MUST be recorded uniformly without provider-specific hardcoding.
