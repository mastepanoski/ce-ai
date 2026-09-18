# Specification: Work Readiness & Verification Advisory

## Formal Requirements

### Requirement 1: ODD Semantic Readiness Assessment
- **WHEN** readiness evaluation is triggered for an Organic Driven Development task (or brief in `odd/tasks/<feature>.md`)
- **THEN** the engine MUST evaluate the 4 ODD semantic dimensions:
  1. `dod_satisfied`: DoD checklist completion and diff alignment.
  2. `guardrails_respected`: Negative constraints and architectural boundaries.
  3. `graduation_recommended`: Scope creep or complexity warranting formal OpenSpec.
  4. `ready_to_close`: Overall task completion readiness.

### Requirement 2: Compound Engineering Stage Readiness Assessment
- **WHEN** readiness evaluation is triggered for a Compound Engineering workflow stage
- **THEN** the engine MUST evaluate stage-appropriate dimensions:
  1. `requirements_clear`: Clarity of scope and acceptance criteria.
  2. `implementation_complete`: Checklist fulfillment.
  3. `tests_sufficient`: Quality gates and behavioral verification.
  4. `documentation_complete`: Docs and architectural alignment.

### Requirement 3: Configurable Integer Percentage Thresholds
- **WHEN** composite readiness score is calculated
- **THEN** it MUST be mapped against configurable thresholds from `state.json` (`readiness.thresholds`):
  - **Ready** (`✓`): `composite_score >= ready_pct / 100.0` (default 80%).
  - **Warning** (`△`): `composite_score >= warning_pct / 100.0` (default 60%).
  - **NotReady** (`⚠`): `composite_score < warning_pct / 100.0`.

### Requirement 4: Advisory Non-Blocking Invariant
- **WHEN** readiness evaluation completes with `Warning` or `NotReady`
- **THEN** the CLI command MUST emit advisory guidance without failing the process or exiting with non-zero exit codes.
- **AND** the Decision Engine MUST NOT mutate `state.json` or force workflow stage transitions.

### Requirement 5: Fail-Safe Non-Disruptive Fallback
- **WHEN** the decision provider is unconfigured, disabled, times out, or returns an error
- **THEN** the evaluation MUST return a fallback result with `fallback_applied = true` and exit code 0.
- **AND** normal developer workflows MUST proceed uninterrupted.

### Requirement 6: ODD Graduation Recommendation Trigger
- **WHEN** `graduation_recommended` dimension confidence exceeds `warning_pct`
- **THEN** the advisory result MUST include an explicit recommendation to run `ce-ai graduate`.

### Requirement 7: CLI Diagnostics & Inspection
- **WHEN** `ce-ai decisions check-readiness` is invoked with `--json`
- **THEN** it MUST output a structured JSON payload containing target, workflow mode, composite score, dimension scores, status, and advisory notes.
- **WHEN** invoked with `--verbose`
- **THEN** it MUST display individual dimension confidence breakdowns.
