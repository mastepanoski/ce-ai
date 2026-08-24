# OpenSpec Design: Release v1.0.0 Architecture

## TUI Modal Text Wrapping Design (`src/tui.rs`)

```rust
let paragraph = Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL).title("Result"))
    .wrap(Wrap { trim: false });
```

## TUI Workflow Direct Dispatch Design (`src/tui.rs`)

```rust
// Handle keypresses in MenuTab::Workflow
match key.code {
    KeyCode::Char('1') => self.stage = WorkflowStage::Brainstorm,
    KeyCode::Char('2') => self.stage = WorkflowStage::OpenSpec,
    KeyCode::Char('3') => self.stage = WorkflowStage::Plan,
    KeyCode::Char('4') => self.stage = WorkflowStage::Work,
    _ => {}
}
```

## Bug Report Template (`.github/ISSUE_TEMPLATE/bug_report.yml`)

```yaml
name: Bug Report
description: File a bug report to help us improve ce-ai
title: "[BUG]: "
labels: ["bug"]
body:
  - type: textarea
    id: description
    attributes:
      label: Bug Description
    validations:
      required: true
  - type: dropdown
    id: os
    attributes:
      label: Operating System
      options:
        - Linux
        - macOS
        - Windows
```
