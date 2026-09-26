# Exploration: Retiring Ratatui TUI & Adopting Lean Terminal Architecture

## Evaluated Alternatives

### Option 1: Status Quo with Full Parity (Rejected)
- **Concept**: Expand `src/tui/` from 15 to 24 tabs to achieve complete parity with all CLI subcommands (`decisions`, `spec`, `doc`, `gate`, `guard`, `report-bug`, `self-update`, `archive`, `graduate`).
- **Why Rejected**:
  - A 24-item vertical menu exceeds standard terminal dimensions (80×24) without nested submenus or scrolling.
  - The fundamental architecture of the TUI is a wrapper around `std::process::Command::new("ce-ai")` (`capture_cli`), which adds unnecessary latency and complexity.
  - AI coding agents and CI runners (the primary users of `ce-ai`) never use interactive screen buffers.
  - The ongoing maintenance tax of updating the TUI on every new command does not justify the negligible human usage.

### Option 2: Optional Cargo Feature (`--features tui`) (Rejected)
- **Concept**: Gate `src/tui/` behind a Cargo feature `tui` so it can be disabled in CI and lightweight builds.
- **Why Rejected**:
  - Introduces `#[cfg(feature = "tui")]` conditionals into `src/lib.rs`, `src/commands/registry.rs`, and `Cargo.toml`.
  - Does not resolve the 9-command parity lag or the subprocess shell paradox.
  - Leaves ~1,835 lines of dead/rarely-used code in the repository that must still be audited and maintained.

### Option 3: Complete Retirement in Favor of Rich Terminal Output (Selected)
- **Concept**: Completely remove `src/tui/` and dependencies `ratatui` and `crossterm`. When `ce-ai` is run with no subcommand, invoke `status::run(ctx)`, displaying an immediate, colored, non-blocking summary of harnesses, version, drift, and project adoption.
- **Why Selected**:
  - **Deterministic Simplicity**: Immediately drops ~1,835 lines of code and eliminates ~8 transitive dependencies.
  - **Zero Terminal Corruption**: Eliminates raw mode, alternate screen buffers, and signal-trapping fragility.
  - **Pipe & Script Friendly**: Bare `ce-ai` works consistently in both interactive terminals and piped subshells without erroring with exit code 2.
  - **Focus on Core Mission**: Aligns 100% with `ce-ai`'s identity as a governance and FSM orchestration layer for AI coding agents.

## Codebase Blast Radius Analysis

### 1. `crossterm` Dependency Scope
- Checked with `git grep "crossterm" src/`:
  - Only referenced in `src/tui/runner.rs`.
  - Password masking in `src/decisions/auth.rs` uses `rpassword::read_password()`, which does not depend on `crossterm`.
  - Pruning `crossterm` has zero side effects outside `src/tui/`.

### 2. `ratatui` Dependency Scope
- Checked with `git grep "ratatui" src/`:
  - Exclusively referenced in `src/tui/handlers.rs`, `src/tui/render.rs`, `src/tui/runner.rs`, and `src/tui/tests/mod_tests.rs`.
  - Pruning `ratatui` has zero side effects outside `src/tui/`.

### 3. Entry Point Dispatch
- In `src/commands/registry.rs`:
  ```rust
  pub fn dispatch(ctx: &Context, command: Option<Commands>) -> Result<(), CeError> {
      match command {
          Some(cmd) => cmd.run(ctx),
          None => status::run(ctx), // Replaces crate::tui::run_interactive(ctx)
      }
  }
  ```
  This is a direct, clean, and backwards-compatible transition.
