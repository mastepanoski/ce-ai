# Requirements Document: Release v1.0.0 — Production Release (Stable API Freeze)

## Problem & Context
With Release v0.9.0 providing 100% test coverage, ISO 27001/27002 security threat matrix validation, and sub-50ms benchmarks, Release `v1.0.0` marks the formal **Production Stable Release** for `ce-ai`. This milestone freezes CLI contracts and schemas, resolves all remaining open TUI and issue template feedback (Issues #72, #75, #76), and completes production user documentation.

---

## Key Requirements & Boundaries

### 1. Stable API Freeze & Configuration Schema Lock (R1)
- **R1.1 Frozen CLI Contract**: Freeze CLI subcommands (`install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, `doctor`, `workflow`, `tools`, `backups`, `tui`) with stable flag compatibility.
- **R1.2 Schema Locking**: Freeze `state.json`, `.ce-ai.json`, and `opencode.json` schemas with strict backward compatibility guarantees.

### 2. TUI Dashboard Polish & Stage Dispatch (Issues #72 & #76) (R2)
- **R2.1 TUI Modal Text Wrapping Fix (#72)**: Fix text wrapping inside `MenuTab::Sync` and `MenuTab::Doctor` result modals so lines do not break mid-word or overflow container bounds.
- **R2.2 Direct Workflow Stage Invocation (#76)**: Allow users to trigger Flywheel workflow stage commands (Brainstorm, Plan, Work, Compound) directly from the `🎮 Workflow (FSM)` panel in `ce-ai tui`.

### 3. Production Governance & Issue Templates (Issue #75) (R3)
- **R3.1 Bug Report Template (#75)**: Create `.github/ISSUE_TEMPLATE/bug_report.yml` for structured bug reporting.
- **R3.2 Production Documentation**: Complete `README.md`, `ROADMAP.md`, `CONCEPTS.md`, and `CHANGELOG.md` for `v1.0.0`.

---

## Out-of-Scope Boundaries (Non-Goals)
- Breaking CLI flag changes (prohibited in v1.0.0).

---

## Success Criteria
1. Frozen CLI command contract and configuration schemas pass 100% unit and integration tests.
2. TUI modal text wrapping fix (Issue #72) renders clean multi-line output within container bounds.
3. TUI direct workflow stage dispatch (Issue #76) executes stage commands cleanly from the FSM panel.
4. `.github/ISSUE_TEMPLATE/bug_report.yml` created and validated.
5. All CI matrix checks pass green across Linux, macOS, and Windows.
