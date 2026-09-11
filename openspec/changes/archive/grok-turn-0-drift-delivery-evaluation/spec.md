# Specification: Grok Build CLI Turn-0 Drift Delivery Evaluation

## Requirements

### R1: Negative Finding Documentation
- WHEN Grok CLI hook capabilities are evaluated,
- THEN the finding MUST be documented in `docs/solutions/architecture/2026-09-02-grok-harness-turn-0-drift-delivery-evaluation.md`.

### R2: Zero Non-Functional Hooks
- WHEN `ce-ai init-prj` adopts a project for Grok,
- THEN it MUST NOT generate non-functional hooks whose stdout is discarded.
- THEN it MUST continue managing `AGENTS.md` and `.grok/rules/compound-engineering.md`.
