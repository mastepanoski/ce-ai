# Proposal: Managed Adoption Block Turn-0 Directives & Progressive OpenSpec Accuracy

## Problem Statement
The managed governance block injected by `ce-ai init-prj` (`render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs`) contains two stale instructions that degrade AI agent efficiency and contradict actual project practices:
1. **Redundant Turn-0 Command Execution**: The current "Turn-0 Session Directives" text unconditionally commands agents to execute `ce-ai workflow resume` at every session start or context compaction. However, `reconcile_project_harness_hooks` (`src/commands/init_prj.rs:486`) already installs native `SessionStart` hooks across supported harnesses (Claude, Cursor, Codex, Copilot, Pi, Antigravity) that automatically execute `ce-ai workflow resume --json` and inject live FSM state before Turn 1. Agents following the block verbatim execute a redundant CLI call whose exact state was already provided in context.
2. **Premature `tasks.md` Requirement**: The current "Stage 2 OpenSpec Enforcement Requirements" block unconditionally requires all 5 OpenSpec artifacts (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) to exist before writing feature code. This contradicts `AGENTS.md` and standard Compound Engineering workflow, where OpenSpec is authored progressively: Stage 2 freezes the contract (proposal, exploration, design, spec), while Stage 3 (`/ce-plan`) derives `tasks.md` from that frozen contract.

## In-Scope
- Update `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs` with accurate, conditional Turn-0 guidance and progressive OpenSpec definition.
- Bump `BLOCK_VERSION` from 6 to 7 in `src/commands/init_prj.rs`.
- Update `CUR_BLOCK_VERSION` and test assertions in `tests/cli.rs` and `src/commands/tests/init_prj.rs`.
- Bump `Cargo.toml` SemVer to `1.72.1` (PATCH) and document in `CHANGELOG.md`.
- Keep `AdoptionTier::Minimal` and `AdoptionTier::Orchestrator` untouched.

## Out-of-Scope
- Changing hook installation mechanics in `reconcile_project_harness_hooks`.
- Modifying `AdoptionTier::Minimal` or `AdoptionTier::Orchestrator` templates.
- Changing `ce-ai workflow resume` CLI flags or output formatting.

## Risks & Mitigations
- **Risk**: Existing projects with `v=6` blocks are marked stale.
  - **Mitigation**: This is the intended behavior of `BLOCK_VERSION`. `ce-ai doctor` and `ce-ai status` cleanly identify `StaleVersion { version: 6 }` and prompt the operator to run `ce-ai init-prj --tier full` to upgrade without loss of custom rules.
- **Risk**: Test regressions in drift detection fixtures.
  - **Mitigation**: Update `CUR_BLOCK_VERSION = 7` in `tests/cli.rs` and synchronize all upgrade fixtures to ensure both generic drift and stale version upgrade paths remain 100% covered.

## Success Criteria
- `render_block_content(AdoptionTier::Full)` emits the updated Turn-0 and Stage 2 text.
- `BLOCK_VERSION` is 7.
- Unit and CLI tests pass 100% (`cargo test`, `make e2e`).
- `ce-ai doc lint --strict` passes.
