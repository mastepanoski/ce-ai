# OpenSpec Exploration: Release v1.0.0 Technical Investigation

## Technical Alternatives Evaluated

### Option A: Ratatui Paragraph Line Wrapping for TUI Modals (Issue #72)
- *Approach*: Configure `ratatui::widgets::Paragraph::wrap(&self, Wrap { trim: false })` using Ratatui's word-wrapper to ensure text fits inside modal borders cleanly.
- *Decision*: Adopt `Wrap { trim: false }` on modal paragraphs in `src/tui.rs`.

### Option B: TUI Direct Workflow Stage Dispatch (Issue #76)
- *Approach*: Map number keys (`1`..`7`) or button actions inside `MenuTab::Workflow` to trigger workflow stage transitions and launch commands in context.
- *Decision*: Implement stage key handlers in `src/tui.rs` for `MenuTab::Workflow`.

### Option C: GitHub Issue Template Schema (Issue #75)
- *Approach*: Add `.github/ISSUE_TEMPLATE/bug_report.yml` following GitHub YAML schema with dropdowns for OS, harness, and reproduction steps.
- *Decision*: Add `.github/ISSUE_TEMPLATE/bug_report.yml`.

---

## Architectural Conclusions
- Freeze CLI contract and configuration schemas with backwards compatibility.
- Zero breaking changes to `state.json` or `.ce-ai.json`.
