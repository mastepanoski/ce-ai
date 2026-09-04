# Specification: Kimi Code CLI Turn-0 Drift Delivery Evaluation

## Requirements

### R1: Negative Finding Documentation
- WHEN Kimi Code CLI hook capabilities are evaluated,
- THEN the finding MUST be documented in `docs/solutions/architecture/2026-09-02-kimi-harness-turn-0-drift-delivery-evaluation.md`.

### R2: Zero Global Contamination on Project Adoption
- WHEN `ce-ai init-prj` adopts a project for Kimi,
- THEN it MUST NOT mutate the user's global `~/.kimi-code/config.toml`.
- THEN it MUST continue managing `AGENTS.md` and `.kimi-code/AGENTS.md`.
