# Technical Execution Plan: Release v1.0.0 — Production Release (Stable API Freeze)

**Release Goal**: Implement TUI text wrapping fix (Issue #72), direct TUI stage dispatch (Issue #76), bug report template (Issue #75), and CLI API freeze for the `v1.0.0` Production Stable Release.

---

## 7-Stage Flywheel Cycle Execution Steps

### Stage 1: Ideation (`ce-brainstorm`) — ✅ COMPLETED
- Brainstormed scope focus for `v1.0.0` Production Stable Release.
- Created `docs/brainstorms/2026-08-21-v100-roadmap-requirements.md`.

### Stage 2: OpenSpec Definition — ✅ COMPLETED
- Created formal specification suite under `openspec/changes/v1_0_0_roadmap/`:
  - `proposal.md`
  - `exploration.md`
  - `design.md`
  - `spec.md`
  - `tasks.md`

### Stage 3: Plan (`ce-plan` & `ce-doc-review`) — 🚧 IN PROGRESS
- Created `docs/plans/2026-08-21-v100-roadmap-execution-plan.md`.

### Stage 4: Work/TDD (`ce-work` & `ce-simplify-code`)
- Unit 1: Fix TUI modal text wrapping (`src/tui.rs`, Issue #72) & add stage shortcut handlers (Issue #76).
- Unit 2: Add `.github/ISSUE_TEMPLATE/bug_report.yml` (Issue #75) and bump version to `1.0.0`.
- Unit 3: Verify CLI contract stability.

### Stage 5: Verify (`cargo test` & `make e2e`)
- Verify zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
- Verify 100% unit & integration test suite (`cargo test`).

### Stage 6: Compound (`ce-compound`)
- Document solution in `docs/solutions/` and update `CONCEPTS.md`.

### Stage 7: Ship (`ce-commit-push-pr`)
- Create feature branch, commit, push, open PR, watch CI, merge to `main`, and publish Release Tag `v1.0.0`.
