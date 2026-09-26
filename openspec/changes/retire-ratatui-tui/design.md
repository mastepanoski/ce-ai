# Design: Retiring Ratatui TUI & Dispatch Simplification

## System Architecture Changes

### 1. Module Layout
The `src/tui/` module tree is removed in its entirety:
```text
src/
├── main.rs            # Unchanged: parses CLI and invokes dispatch
├── lib.rs             # Remove `pub mod tui;`
├── commands/
│   ├── registry.rs    # Update `dispatch`: `None => status::run(ctx)`
│   ├── status.rs      # Unchanged: handles rich terminal status
│   └── ...
├── harness/           # Unchanged
├── state/             # Unchanged
└── tui/               # [DELETED] All 8 files removed (~1,835 LOC)
    ├── app.rs
    ├── handlers.rs
    ├── mod.rs
    ├── render.rs
    ├── runner.rs
    ├── spawn.rs
    ├── tabs.rs
    └── tests/mod_tests.rs
```

### 2. Dependency Manifest (`Cargo.toml`)
Prune terminal rendering dependencies:
```toml
# REMOVED:
# ratatui = "0.30"
# crossterm = "0.28"
```
This cascades into `Cargo.lock`, dropping:
- `ratatui`, `ratatui-core`, `ratatui-crossterm`, `ratatui-macros`, `ratatui-termina`, `ratatui-termwiz`, `ratatui-widgets`
- `crossterm`, `crossterm_winapi`
- `lru` (associated with ratatui)

### 3. CLI Dispatch Contract (`src/commands/registry.rs`)
```rust
/// Registry dispatch — thin wrapper used by `main.rs`.
pub fn dispatch(ctx: &Context, command: Option<Commands>) -> Result<(), CeError> {
    match command {
        Some(cmd) => cmd.run(ctx),
        None => status::run(ctx),
    }
}
```
When invoked without arguments:
- Prints installed harnesses and detected versions.
- Prints drift status.
- Prints adopted project status.
- Prints git working tree state and gate check summary.
- Exits with `0` (Success) instead of erroring in non-TTY environments.

### 4. Documentation Changes
- Remove `docs/user-guide/workflow-panel-native-vs-agent-skills.md`.
- Remove link to workflow panel from `README.md`.
- Clean up any incidental references to the full-screen dashboard across user guides.
