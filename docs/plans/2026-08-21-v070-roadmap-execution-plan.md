# Technical Execution Plan: Release v0.7.0 — Workspace Overrides & Multi-Harness Uninstall Parity

**Release Goal**: Implement repository-local `.ce-ai.json` overrides with precedence resolution, and close multi-harness parity for `ce-ai uninstall --harness all --all` (Issue #64).

---

## 7-Stage Flywheel Cycle Execution Steps

### Stage 1: Ideation (`ce-brainstorm`) — ✅ COMPLETED
- Brainstormed dual scope: Workspace configuration overrides (`.ce-ai.json`) + Complete multi-harness uninstallation (`ce-ai uninstall --harness all --all`).

### Stage 2: OpenSpec Definition — ✅ COMPLETED
- Created formal specification suite under `openspec/changes/v0_7_0_roadmap/`:
  - `proposal.md`
  - `exploration.md`
  - `design.md`
  - `spec.md`
  - `tasks.md`

### Stage 3: Plan (`ce-plan` & `ce-doc-review`) — 🚧 IN PROGRESS
- Created `docs/plans/2026-08-21-v070-roadmap-execution-plan.md`.
- Perform doc review pass (`ce-doc-review`).

### Stage 4: Work/TDD (`ce-work` & `ce-simplify-code`)
- Unit 1: Implement `State::load_with_workspace_overrides` in `src/state/state.rs`.
- Unit 2: Implement multi-harness uninstall with `--harness all --all --yes` in `src/commands/uninstall.rs`.
- Unit 3: Add CLI integration tests in `tests/cli.rs`.
- Refactor with `ce-simplify-code` and code review with `ce-code-review`.

### Stage 5: Verify (`cargo test` & `make e2e`)
- Verify zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- Verify 100% unit & integration test suite (`cargo test`).
- Run containerized Docker E2E gate (`make e2e`).

### Stage 6: Compound (`ce-compound`)
- Document learnings in `docs/solutions/` and update `CONCEPTS.md`.

### Stage 7: Ship (`ce-commit-push-pr`)
- Create feature branch, commit, push, open PR, watch CI, and merge to `main`.
