# Proposal: Retire Ratatui TUI in Favor of Lightweight Rich Terminal Summary

## Problem Statement
`ce-ai` has matured from an early visual plugin manager into an enterprise-grade workflow orchestration and governance layer for AI coding agents (Claude Code, Cursor, OpenCode, Codex, Copilot, AGY, Pi, etc.) and automated CI/CD pipelines.

The embedded full-screen Ratatui TUI dashboard in `src/tui/` (~1,835 LOC) has become an architectural liability:
1. **Target Audience Mismatch**: AI coding agents operate via non-interactive subshells, capturing stdout/stderr and JSON exit codes. They never use interactive full-screen TUIs, and in non-TTY environments bare `ce-ai` fails with a usage error.
2. **Chronic Parity Lag**: The CLI has expanded to 24 subcommands, while the TUI is permanently lagging with 15 tabs (missing `decisions`, `spec`, `doc`, `gate`, `guard`, `report-bug`, `self-update`, `archive`, `graduate`). Fitting 24 items in an 80×24 Ratatui sidebar breaks layout and usability.
3. **Subprocess Shell Paradox**: `src/tui/handlers.rs` does not call Rust domain methods directly; it spawns child `ce-ai` processes (`capture_cli`) and renders stdout into modal popups.
4. **Heavy Dependency & Vulnerability Footprint**: `ratatui = "0.30"` and `crossterm = "0.28"` pull in ~8 transitive crates (`lru`, `termwiz`, `crossterm_winapi`, etc.), which previously triggered Dependabot vulnerability alerts.
5. **Terminal State Fragility**: Managing raw-mode and alternate screen buffers carries a continuous risk of leaving the user's terminal corrupted upon an unexpected panic or signal.

## In-Scope
1. **Retire TUI Subsystem**:
   - Delete `src/tui/` directory and its module declarations (`src/lib.rs`, `src/tui/`).
   - Remove `ratatui` and `crossterm` dependencies from `Cargo.toml`.
2. **Rich Default Terminal Overview**:
   - Update `src/commands/registry.rs:dispatch`: When `ce-ai` is run with no subcommand, instead of attempting to enter raw mode or erroring, display a clean, non-blocking status overview (reusing `status::run(ctx)` or enhanced overview) accompanied by quick command guidance.
3. **Documentation Hygiene**:
   - Remove obsolete TUI guide (`docs/user-guide/workflow-panel-native-vs-agent-skills.md`).
   - Update `README.md`, architecture guides, and user documentation to remove TUI references and reflect the streamlined, agent-first CLI architecture.
   - Update `CHANGELOG.md` documenting the TUI retirement and dependency pruning.
4. **Verification & Quality Gates**:
   - Ensure all existing unit, integration, and security tests pass.
   - Verify `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check`.

## Out-of-Scope
- Modifying the behavior or arguments of any existing CLI subcommand (`install`, `sync`, `doctor`, `models`, `workflow`, etc.).
- Modifying interactive stdin prompts in `ce-ai report-bug` (which uses standard stdin/stdout) or `ce-ai decisions auth` (which uses `rpassword`).

## Risk Evaluation
- **Risk**: Existing scripts or users who typed bare `ce-ai` might be surprised by the change.
  - *Mitigation*: Bare `ce-ai` will now cleanly print the system and project status with recommended next actions (identical to `ce-ai status`), which is much more informative, fast, and pipe-friendly.
- **Risk**: Regression in tests that expected TUI modules.
  - *Mitigation*: All tests in `tests/cli.rs` invoke explicit subcommands; TUI tests were self-contained in `src/tui/tests/`. Any TUI-specific tests will be cleanly retired.

## Success Criteria
1. `src/tui/` is removed, reducing codebase size by ~1,835 LOC.
2. `ratatui` and `crossterm` are completely purged from `Cargo.toml` and `Cargo.lock`.
3. Running bare `ce-ai` in any terminal (interactive or non-interactive pipe) prints a clean, fast status overview without errors or screen clears.
4. 100% green test matrix across all platforms.
