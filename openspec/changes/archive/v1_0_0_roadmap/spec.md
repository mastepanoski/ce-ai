# OpenSpec Requirements: Release v1.0.0 Specifications

## Feature 1: Stable API & Configuration Schema Freeze

### Requirement 1.1: CLI Contract Stability
- **WHEN** any `ce-ai` subcommand (`install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, `doctor`, `workflow`, `tools`, `backups`, `tui`) is invoked with established flags,
- **THEN** execution MUST maintain full backwards compatibility without breaking flag signatures.

---

## Feature 2: TUI Modal Text Wrapping & Workflow Stage Dispatch (Issues #72 & #76)

### Requirement 2.1: TUI Modal Text Wrapping (Issue #72)
- **WHEN** a result modal in `ce-ai tui` displays long diagnostic or sync output,
- **THEN** text MUST wrap cleanly at word boundaries using `Wrap { trim: false }` without overflowing container borders.

### Requirement 2.2: Workflow Stage Invocation (Issue #76)
- **WHEN** the user presses stage shortcut keys in `MenuTab::Workflow`,
- **THEN** the active Flywheel stage MUST update and launch the corresponding stage transition cleanly.

---

## Feature 3: Bug Report Template (Issue #75)

### Requirement 3.1: Bug Report Form
- **WHEN** opening a bug report on GitHub,
- **THEN** `.github/ISSUE_TEMPLATE/bug_report.yml` MUST present structured fields for OS, harness, description, and reproduction steps.
