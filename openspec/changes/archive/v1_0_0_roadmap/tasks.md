> STATUS (v1.20.1): Superseded: releases >= v1.0 shipped. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Release v1.0.0 Implementation Plan

- [ ] **Unit 1: TUI Modal Text Wrapping & Workflow Stage Dispatch (Issues #72 & #76)**
  - [ ] Implement `Wrap { trim: false }` for result modals in `src/tui.rs` (Issue #72).
  - [ ] Implement direct stage dispatch key handlers in `src/tui.rs` (Issue #76).

- [ ] **Unit 2: Issue Templates & Production Governance (Issue #75)**
  - [ ] Create `.github/ISSUE_TEMPLATE/bug_report.yml` (Issue #75).
  - [ ] Update `Cargo.toml`, `Formula/ce-ai.rb`, `README.md`, `CHANGELOG.md`, `ROADMAP.md`, and `CONCEPTS.md` for `v1.0.0`.

- [ ] **Unit 3: Stable API Freeze & Final Verification**
  - [ ] Verify zero breaking changes to CLI flags and configuration schemas.
  - [ ] Run full test suite and Docker E2E gate.
