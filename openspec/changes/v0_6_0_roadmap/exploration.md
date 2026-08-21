# OpenSpec Exploration: Release v0.6.0 Roadmap

## Technical Investigation

### Option 1: Full-Screen TUI FSM Tab
- **Evaluated**: Render Ratatui `Block`, `Gauge`, and `Table` widgets inside `src/tui.rs` mapping to `FsmState` stored in `state.json`.
- **Trade-offs**: Low resource footprint; zero additional binary dependencies.

### Option 2: Extended Companion Diagnostics
- **Evaluated**: Read SQLite header / run `PRAGMA quick_check;` on `~/.engram/engram.db` (or test connection), query `codegraph status` JSON output if available, and verify `rtk` binary on PATH.
- **Trade-offs**: Provides deep empirical diagnostic confidence for `ce-ai doctor` without needing unsafe C-bindings.

### Option 3: Real-Time Sync Watcher
- **Evaluated**: Use file modification timestamps or a lightweight watcher loop in `src/commands/sync.rs` (`ce-ai sync --watch`) to trigger `write_atomic` when managed skills change.
- **Trade-offs**: Prevents drift immediately when users edit local skills.
