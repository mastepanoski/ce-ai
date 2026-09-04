# Design: Kimi Code CLI Turn-0 Drift Delivery Evaluation

## Architectural Decision
Record negative finding for Kimi Code CLI Turn-0 hook injection in `docs/solutions/architecture/2026-09-02-kimi-harness-turn-0-drift-delivery-evaluation.md`.

## Preservation of Current Behavior
- Do not implement `has_session_start_hook` / `ensure_session_start_hook` in `src/harness/kimi.rs`.
- Retain existing `AGENTS.md` and `.kimi-code/AGENTS.md` managed directive block in `src/commands/init_prj.rs`.
