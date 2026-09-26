---
title: "Retiring Ratatui TUI Dashboard in Favor of Lean Terminal Architecture"
category: "architecture"
date: "2026-09-26"
tags:
  - tui-retirement
  - cli-dispatch
  - dependency-pruning
  - agent-first-architecture
  - zero-terminal-corruption
applies_when: "When architecting CLI dispatch entry points, evaluating interactive TUI vs head-first terminal output, or pruning legacy UI subsystems."
problem_type: "architectural_refactor"
---

# Retiring Ratatui TUI Dashboard in Favor of Lean Terminal Architecture

## Context & Problem
In early iterations of `ce-ai`, a full-screen Ratatui/Crossterm TUI dashboard (`src/tui/`) was built to provide human developers with an interactive terminal interface for browsing harnesses, models, and triggering commands.

As `ce-ai` matured into a deterministic workflow orchestration and governance layer for **AI coding agents** (Claude Code, Cursor, OpenCode, Codex, Copilot, AGY, Pi, etc.) and CI/CD pipelines, the TUI became an architectural liability:
1. **Target Audience Mismatch**: AI agents invoke CLI commands via non-interactive subshells and parse stdout/stderr and JSON exit codes. In non-TTY environments, bare `ce-ai` failed with a usage error (`exit code 2`).
2. **Chronic Parity Lag**: The CLI grew to 24 subcommands, while the TUI was frozen at 15 tabs. Fitting 24 commands into an 80×24 Ratatui sidebar was impossible without cramped multi-level menus.
3. **Subprocess Shell Paradox**: The legacy TUI handlers (`<src/tui/handlers.rs>`) did not invoke domain methods; they executed `std::process::Command::new("ce-ai")` (`capture_cli`) and rendered the captured output inside modal popups.
4. **Dependency & Attack Surface**: `ratatui` (0.30) and `crossterm` (0.28) pulled in ~8 transitive dependencies (`lru`, `termwiz`, `crossterm_winapi`), which previously triggered Dependabot vulnerability alerts.
5. **Terminal State Fragility**: Required raw-mode and alternate-screen buffer management with `RawModeGuard`, risking corrupted terminal state on unexpected panics or signals.

## Solution Architecture
1. **Subsystem Removal**:
   - Safely deleted `src/tui/` (~1,835 LOC) across 8 files (`app.rs`, `handlers.rs`, `mod.rs`, `render.rs`, `runner.rs`, `spawn.rs`, `tabs.rs`, and tests).
   - Removed `pub mod tui;` from `src/lib.rs`.
   - Purged `ratatui` and `crossterm` from `Cargo.toml` and `Cargo.lock`.
2. **Deterministic Bare CLI Routing**:
   - In `src/commands/registry.rs`, updated `dispatch`:
     ```rust
     pub fn dispatch(ctx: &Context, command: Option<Commands>) -> Result<(), CeError> {
         match command {
             Some(cmd) => cmd.run(ctx),
             None => status::run(ctx),
         }
     }
     ```
   - Running `ce-ai` without arguments now immediately prints a clean, rich status overview of installed harnesses, drift status, adopted project rules, and git state, exiting with `0` (Success) in both interactive and non-interactive environments.
3. **Documentation Alignment**:
   - Removed obsolete guide `docs/user-guide/workflow-panel-native-vs-agent-skills.md`.
   - Updated `README.md`, `AGENTS.md`, and user guides to reflect the streamlined agent-first CLI.

## Verification & Benefits
- Codebase reduced by ~1,835 lines of code.
- Dependencies reduced by 11 packages in `Cargo.lock`.
- `ce-ai` now executes instantaneously (<50ms) on bare invocation without taking over the terminal screen or risking raw-mode corruption.
- All 215 tests pass green.
