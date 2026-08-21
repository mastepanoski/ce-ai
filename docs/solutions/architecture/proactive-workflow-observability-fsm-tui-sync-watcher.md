---
title: "Proactive Workflow Observability: TUI FSM Dashboard, Extended Doctor Health, and Real-Time Sync Watcher"
category: "architecture"
date: "2026-08-21"
tags:
  - fsm-engine
  - tui-dashboard
  - companion-health
  - sync-watcher
  - workflow-observability
components:
  - tui
  - doctor
  - sync
  - workflow
applies_when: "Adding workflow observability, live progress gauges in TUI, extended companion health diagnostics, or real-time sync watchers to ce-ai"
---

# Proactive Workflow Observability: TUI FSM Dashboard, Extended Doctor Health, and Real-Time Sync Watcher

## Context

While `ce-ai` v0.5.0 established workspace scope isolation (`--scope workspace`), companion tools management (`ce-ai tools`), and progress checkpointing (`ce-ai workflow`), AI agents and developers previously lacked real-time visibility into the FSM state unless they manually executed CLI commands.

Release v0.6.0 introduces **Proactive Workflow Observability**, transforming `ce-ai` into a live flight cockpit for AI coding assistants.

---

## Guidance & Architecture Patterns

### 1. Interactive TUI Workflow Dashboard (`src/tui.rs`)
- **Pattern**: Add a dedicated `🎮 Workflow (FSM)` tab to `MenuTab` in `src/tui.rs`.
- **Behavior**: Reads `state.json` on each render pass to display:
  - The active 7-stage Flywheel stage (`Ideation`, `OpenSpec`, `Plan`, `Work`, `Verify`, `Compound`, `Ship`).
  - Active subtask string and latest progress checkpoint timestamp.
- **Mental Model**: Acts like an airplane cockpit instrument display—giving immediate visual feedback on agent progress.

### 2. Extended Doctor Health Diagnostics (`src/commands/doctor.rs`)
- **Pattern**: Non-fatal informational health probes added to `ce-ai doctor`:
  - `doctor_check_engram_db()`: Verifies existence and read permissions for `~/.engram/engram.db`.
  - `doctor_check_codegraph_index()`: Verifies `.codegraph/` index presence in the active repository root.
  - `doctor_check_rtk_path()`: Probes system PATH for the `rtk` (Rust Token Killer) binary.
- **Mental Model**: Acts like a pre-flight maintenance check before launching long-running agent tasks.

### 3. Real-Time Sync Watcher (`src/commands/sync.rs` & `src/main.rs`)
- **Pattern**: Add `--watch` flag parsing in Clap for `ce-ai sync`.
- **Behavior**: Runs an initial reconciliation pass, then continuously monitors managed skill paths (`.opencode/plugins`, `.opencode/skills`, `.claude/`), re-syncing SHA256 integrity upon detecting file mutations.
- **Mental Model**: Acts like an autopilot guardrail—preventing configuration drift across multiple host harnesses.

---

## Why This Matters

1. **Zero Context Loss**: Visualizing progress checkpoints in TUI prevents agents from losing context after session compactions.
2. **Preventative Maintenance**: Detecting missing Engram databases or un-initialized CodeGraph indexes before running tasks prevents agent failures.
3. **Automated Drift Elimination**: The `--watch` mode guarantees that edits made in one harness (e.g. Cursor) immediately propagate across all installed harnesses (e.g. Claude Code, Antigravity, OpenCode).

---

## When to Apply

- **TUI Dashboard**: Use whenever operating in interactive terminal mode (`ce-ai tui`) to monitor live FSM progress.
- **Doctor Diagnostics**: Run `ce-ai doctor` before initiating complex multi-stage tasks or release builds.
- **Sync Watcher**: Run `ce-ai sync --watch` in background developer sessions when actively editing local skill definitions or custom plugins.

---

## Examples & Code Snippets

### TUI Workflow Tab Match Arm (`src/tui.rs`):
```rust
MenuTab::Workflow => {
    let mut lines = vec![
        Line::from(Span::styled("Workflow FSM Engine & Progress Recovery:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("7-Stage Flywheel Cycle:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  • [1: Ideation]   ➔ ce-brainstorm / ce-ideate / ce-strategy"),
        Line::from("  • [2: OpenSpec]   ➔ Formal Spec Definition (proposal, spec, tasks)"),
        Line::from("  • [3: Plan]       ➔ ce-plan / ce-doc-review"),
        Line::from("  • [4: Work/TDD]   ➔ ce-work / ce-debug / ce-simplify-code"),
        Line::from("  • [5: Verify]     ➔ Empirical Testing (cargo test, make e2e)"),
        Line::from("  • [6: Compound]   ➔ ce-compound (docs/solutions/)"),
        Line::from("  • [7: Ship]       ➔ ce-commit-push-pr"),
    ];
    // Renders latest checkpoint from state.json
}
```

### Extended Doctor Probes (`src/commands/doctor.rs`):
```rust
if let Ok(home) = std::env::var("HOME") {
    let engram_db = std::path::Path::new(&home).join(".engram").join("engram.db");
    if !engram_db.exists() {
        println!("doctor-info: engram db (~/.engram/engram.db) not found");
    }
}
```

### Sync Watcher CLI Invocation (`src/commands/sync.rs`):
```bash
ce-ai sync --watch
```
