# Specification: Retiring Ratatui TUI & Dispatch Simplification

## Requirements

### Requirement 1: Complete Removal of TUI Subsystem
- **WHEN** the project is compiled,
- **THEN** no source files under `src/tui/` shall exist, and `src/lib.rs` shall not expose `pub mod tui`.

### Requirement 2: Dependency Manifest Pruning
- **WHEN** dependencies in `Cargo.toml` and `Cargo.lock` are audited,
- **THEN** neither `ratatui` nor `crossterm` shall be present in the dependency tree.

### Requirement 3: Deterministic Bare CLI Dispatch
- **WHEN** `ce-ai` is executed without any subcommand in an interactive terminal or in a non-interactive pipe,
- **THEN** it shall execute the status command (`status::run(ctx)`), outputting system status to `stdout` and exiting with exit code `0` (Success).

### Requirement 4: Help and Usage Preservation
- **WHEN** `ce-ai --help` or `ce-ai -h` is executed,
- **THEN** it shall output standard Clap CLI help documentation without referencing a TUI mode.

### Requirement 5: Documentation Consistency
- **WHEN** documentation in `README.md` and `docs/` is checked,
- **THEN** all references to the obsolete full-screen TUI dashboard shall be removed, and `docs/user-guide/workflow-panel-native-vs-agent-skills.md` shall be deleted.

## Acceptance Criteria
1. `cargo check` and `cargo build` compile without warnings.
2. `cargo test` passes 100% of tests.
3. Running `ce-ai` without arguments in terminal prints status output cleanly to stdout and exits with code 0.
4. `Cargo.toml` contains no references to `ratatui` or `crossterm`.
