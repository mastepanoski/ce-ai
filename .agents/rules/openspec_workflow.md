# RULE: Mandatory OpenSpec Development Cycle Enforcement

## Directives for AI Agents

1. **OpenSpec Required Before Code**:
   - Every feature, architectural refactor, or new capability MUST have an active or updated spec inside `openspec/changes/<feature_name>/`.
   - Never generate feature code or pull requests without checking for or creating the corresponding `openspec` files (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).

2. **Strict 7-Stage Cycle Execution**:
   - **Stage 1 (Ideation)**: Frame requirements (`ce-brainstorm` / `ce-ideate`).
   - **Stage 2 (OpenSpec)**: Write/update `openspec/changes/<feature>/` (Mandatory).
   - **Stage 3 (Plan)**: Create task execution plan (`ce-plan`).
   - **Stage 4 (TDD & Build)**: Implement using Red-Green-Refactor (`ce-work` / `ce-debug`).
   - **Stage 5 (Verify)**: Run `cargo fmt`, `clippy`, `cargo test`, `make e2e`, and CI checks.
   - **Stage 6 (Compound)**: Persist durable learnings in `docs/solutions/` (`ce-compound`).
   - **Stage 7 (Ship)**: Commit with clear value message and open PR (`ce-commit-push-pr`).

3. **Enforcement Checklist**:
   - [ ] `openspec/changes/<feature_name>/` contains `proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`.
   - [ ] Unit & E2E tests pass 100%.
   - [ ] Definition of Done criteria in `AGENTS.md` fully satisfied.
