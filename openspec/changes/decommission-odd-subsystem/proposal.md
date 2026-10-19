# Proposal: Decommission ODD Subsystem and Align Compound Agile Fast-Path

## Problem Statement

The **Organic Driven Development (ODD Fast-Path)** subsystem was originally adapted into `ce-ai` to provide a lightweight alternative to full OpenSpec changes for small tasks (< 200 LOC). However, in practice:
1. **Total Orphan Artifact**: Zero files in `odd/tasks/` have ever been created in real-world development across the entire history of `ce-ai` and consumer projects.
2. **Ecosystem Disconnect**: Upstream Compound Engineering skills (`/ce-plan`, `/ce-work`, `/ce-debug`) do not recognize `odd/tasks/`. Instead, they natively use `docs/plans/` or execute direct prompts.
3. **Semantic Overreach & Friction**: Forcing an artificial `odd/tasks/<feature>.md` file structure and `workflow graduate` command violates the core principle of **No Semantic Authority** (Invariant 6 of `AGENTS.md`), while confusing developers with misleading `workflow status` banners.

## In-Scope
- Re-align the fast-path execution mode in `ce-ai workflow status` and `workflow resume` to reflect native Compound Engineering agile execution (`docs/plans/`, `/ce-work`, `/ce-debug`).
- Deprecate `ce-ai workflow graduate` and the `odd/tasks/` directory structure with helpful migration and guidance notices while preserving backward-compatible execution for existing fixtures.
- Update `ce-ai workflow guide` to present the **Compound Agile Fast-Path** alongside the 7-stage Compound Engineering cycle and the OpenSpec Rule of Thumb.
- Realign documentation (`docs/user-guide/odd-fast-path-and-graduation-masterclass.md`, `ce-ai-positioning.md`, `cli-reference.md`, `README.md`, `CONCEPTS.md`) and author an architectural solution document.

## Out-of-Scope
- Deleting the `ExecutionMode::Organic` enum variant from `src/state/state.rs` (it must remain for backward-compatible deserialization of historical `state.json` files).
- Modifying the observe-only 200 LOC ceiling heuristic in `src/commands/gate.rs` (the LOC check remains a valuable advisory metric).

## Risk Evaluation
- **State Compatibility**: Existing `state.json` configurations with `"execution_mode": "organic"` will continue to deserialize safely and map cleanly to the agile fast path.
- **CLI Invariant**: The `graduate` command remains callable for legacy fixtures with a non-blocking deprecation notice.

## Success Criteria
1. `ce-ai workflow status` displays the **Compound Agile Fast-Path** banner without referencing `odd/tasks/` or `graduate`.
2. `ce-ai workflow graduate` emits an informative deprecation notice pointing to `docs/plans/` and `openspec/changes/`.
3. All existing unit, integration, and CLI tests pass without regression.
4. User guides and `CONCEPTS.md` are accreted monotonically and verified via `cargo run -- doc lint --strict`.
