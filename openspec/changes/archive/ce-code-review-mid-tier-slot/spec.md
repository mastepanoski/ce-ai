# Specification: ce-code-review Mid-Tier Model Slot

## Requirements

### Requirement 1: Slot Persistence & Invariants
- **WHEN** a user executes `ce-ai models set --harness opencode ce-code-review-mid-tier <provider/model>`,
- **THEN** `ce-ai` MUST persist the assignment to `state.json` (`model_assignments.ce-code-review-mid-tier`).
- **THEN** `ce-ai` MUST persist the assignment to `opencode.json` under `agent.ce-code-review-mid-tier.model`.
- **THEN** it MUST NOT write a `variant` key under `agent.ce-code-review-mid-tier`.
- **THEN** it MUST perform mutations using `write_atomic`.
- **THEN** it MUST append a snapshot to the profiles directory recording before and after states.

### Requirement 2: Distinguished Display in `ce-ai models list`
- **WHEN** `ce-ai models list` executes,
- **THEN** it MUST display `ce-code-review-mid-tier` clearly distinguishable as a tiering sub-slot rather than an independent top-level stage slot.
- **WHEN** both `ce-code-review` and `ce-code-review-mid-tier` are configured,
- **THEN** `ce-code-review-mid-tier` MUST be rendered nested directly beneath `ce-code-review` (e.g. `  └─ mid-tier (ce-code-review-mid-tier): <provider/model>`).
- **WHEN** `ce-code-review-mid-tier` is configured but `ce-code-review` is not,
- **THEN** it MUST be rendered with explicit sub-slot labeling (e.g. `ce-code-review-mid-tier (mid-tier sub-slot): <provider/model>`).

### Requirement 3: Doctor Health Diagnostic Note
- **WHEN** `ce-ai doctor` runs on an OpenCode installation where `ce-code-review` has an assigned model but `ce-code-review-mid-tier` is unset,
- **THEN** it MUST emit an informational (non-blocking) diagnostic matching:
  `doctor-info: ce-code-review has a model assigned but 'ce-code-review-mid-tier' is not configured; persona-level cost tiering has no explicit target on opencode and may silently fall back to the session model (run 'ce-ai models set --harness opencode ce-code-review-mid-tier <provider/model>' to configure)`.
- **THEN** this diagnostic MUST NOT be treated as an error finding or cause `ce-ai doctor` to exit with a non-zero code.
- **WHEN** `ce-code-review` does not have a model assigned, or `ce-code-review-mid-tier` is configured,
- **THEN** `ce-ai doctor` MUST NOT emit this diagnostic note.

### Requirement 4: Unset Behavior and Opt-In Monotonicity
- **WHEN** no mid-tier slot is set,
- **THEN** runtime behavior MUST remain unchanged from existing behavior: `ce-ai` MUST NEVER fabricate, guess, or write a default model for `ce-code-review-mid-tier`.
- **THEN** `ce-ai` MUST NOT modify or intercept the compound-engineering plugin's internal dispatch files.
