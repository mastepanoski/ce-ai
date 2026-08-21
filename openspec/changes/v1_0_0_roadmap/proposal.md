# OpenSpec Proposal: Release v1.0.0 — Production Release (Stable API Freeze)

## Problem Statement
`ce-ai` has reached production maturity across 12 AI coding agent harnesses with multi-platform binary compilation, universal installers, workspace overrides, ISO 27001/27002 security threat matrix validation, and sub-50ms benchmarks. Release `v1.0.0` represents the formal **Production Stable Release**. It freezes the CLI contract and configuration schemas, addresses open TUI and issue template feedback (Issues #72, #75, #76), and delivers production documentation.

## In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **Stable API & Schema Freeze (R1)**: Lock CLI command flags and configuration schemas (`state.json`, `.ce-ai.json`, `opencode.json`).
- **TUI Modal Text Wrapping Fix (Issue #72)**: Fix text wrapping in `MenuTab::Sync` and `MenuTab::Doctor` result modals.
- **TUI Direct Stage Invocation (Issue #76)**: Allow direct execution of 7-stage Flywheel workflow commands from the TUI Workflow (FSM) dashboard.
- **Bug Report Template (Issue #75)**: Add `.github/ISSUE_TEMPLATE/bug_report.yml`.
- **Production Documentation**: Complete `README.md`, `ROADMAP.md`, `CONCEPTS.md`, and `CHANGELOG.md`.

### Out-of-Scope:
- Breaking CLI command or flag changes.

## Success Criteria
1. Frozen CLI contract and configuration schemas pass 100% integration tests.
2. TUI modal text wrapping fix (Issue #72) renders clean multi-line output within container bounds.
3. TUI direct stage dispatch (Issue #76) executes stage commands cleanly.
4. Issue #75 bug report template passes YAML syntax validation.
5. All CI matrix checks pass green across Linux, macOS, and Windows.
