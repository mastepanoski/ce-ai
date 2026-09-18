# Specification: Risk-Aware Tool Execution (Intelligent Permission & Risk Engine)

## Requirements

### R1: Deterministic Security Rule Precedence
- **WHEN** a tool execution request matches a prohibited deterministic pattern (e.g. root filesystem destruction, privilege escalation, protected path traversal, credential tampering)
- **THEN** the engine MUST immediately return `ExecutionPolicy::Deny`.
- **THEN** this decision MUST NOT be overridden or weakened by any probabilistic provider response.

### R2: Semantic Risk Dimension Classification
- **WHEN** an ambiguous tool operation requires evaluation and risk evaluation is enabled (`enabled = true`)
- **THEN** the system MUST construct a `DecisionRequest` evaluating 6 risk dimensions:
  `destructive`, `credential_sensitive`, `external_side_effect`, `privilege_escalation`, `irreversible`, `scope_exceeds_task`.
- **WHEN** the Decision Engine returns confidence scores for these questions
- **THEN** the composite risk score MUST be calculated from the evaluated dimensions.

### R3: Configurable Integer-Percentage Thresholding
- **WHEN** configuring risk thresholds in `state.json` (`[decisions.risk.thresholds]`)
- **THEN** thresholds MUST be stored as integer percentages (`confirmation_threshold_pct: u32 = 60`, `deny_threshold_pct: u32 = 90`).
- **WHEN** the composite risk score meets or exceeds `deny_threshold_pct`
- **THEN** the system MUST return `ExecutionPolicy::Deny`.
- **WHEN** the composite risk score meets or exceeds `confirmation_threshold_pct` but is below `deny_threshold_pct`
- **THEN** the system MUST return `ExecutionPolicy::RequireConfirmation`.
- **WHEN** the composite risk score is below `confirmation_threshold_pct`
- **THEN** the system MUST return `ExecutionPolicy::Allow`.

### R4: Conservative Fail-Closed Degradation
- **WHEN** risk evaluation is enabled but the decision provider times out, encounters an HTTP network error, or trips the circuit breaker
- **THEN** the system MUST NOT silently allow the operation.
- **THEN** the system MUST return the configured conservative fallback policy (default: `ExecutionPolicy::RequireConfirmation`) with `fallback_applied = true`.

### R5: Sensitive Argument Redaction & Audit Logging
- **WHEN** commands or tool arguments are evaluated or recorded
- **THEN** the engine MUST sanitize known sensitive patterns (API keys, private keys, passwords) replacing them with `[REDACTED]`.
- **WHEN** risk evaluations are executed
- **THEN** an audit event MUST be written to the local configuration log (`risk-events.jsonl`).

### R6: CLI Subcommand Diagnostics
- **WHEN** executing `ce-ai decisions check-risk "<tool>" "<command>"`
- **THEN** `ce-ai` MUST print human-readable policy outcomes (`Allow`, `RequireConfirmation`, `Deny`) with rationale and risk scores.
- **WHEN** the `--json` flag is provided
- **THEN** `ce-ai` MUST output structured JSON containing `tool`, `command`, `policy`, `reason`, `composite_risk_score`, `dimensions`, `fallback_applied`, and `latency_ms`.
- **WHEN** the `--verbose` flag is provided
- **THEN** `ce-ai` MUST display per-dimension confidence scores and threshold details.

### R7: Setup Preset & Doctor Probe Integration
- **WHEN** running `ce-ai decisions setup --preset recommended|shadow|local`
- **THEN** `state.decisions.risk` MUST be provisioned with standard thresholds (60% confirmation, 90% deny).
- **WHEN** `ce-ai doctor` is executed
- **THEN** it MUST inspect `state.decisions.risk` and report the active risk evaluation status, thresholds, and fallback policy.

---

## Acceptance Criteria

1. **Deterministic Invariant Test**: Deterministic denials (e.g. `rm -rf /`, `sudo su`, access to `.ssh/id_rsa`) are denied with `ExecutionPolicy::Deny` even if a mock decision provider claims the operation is 100% safe.
2. **Threshold Sensitivity Test**: Unit tests verify operations crossing 60% trigger `RequireConfirmation` and crossing 90% trigger `Deny`.
3. **Fail-Closed Verification**: Simulated provider timeouts or 500 errors fail closed to `RequireConfirmation` rather than `Allow`.
4. **Redaction Check**: Tokens such as `sk-1234567890abcdef12345678` are verified to be replaced with `[REDACTED]` in decision prompts and audit records.
5. **Quality Gates**: Zero Clippy warnings, 100% test pass on `cargo test`, and 100% green CI matrix.
