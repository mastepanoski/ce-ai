# Design: Grok Build CLI Turn-0 Drift Delivery Evaluation

## Architectural Decision
Record negative finding for Grok Build CLI Turn-0 hook injection in `docs/solutions/architecture/2026-09-02-grok-harness-turn-0-drift-delivery-evaluation.md`.

## Preservation of Current Behavior
- Do not implement `has_session_start_hook` / `ensure_session_start_hook` in `src/harness/grok.rs`.
- Retain existing `AGENTS.md` and `.grok/rules/compound-engineering.md` managed directive block.
