# OpenSpec Design: Release v0.6.0 Roadmap

## Architecture & Data Schemas

### 1. TUI Workflow Dashboard (`src/tui.rs`)
- Add a new tab `Workflow (FSM)` to the TUI navigation bar `[ Install | Status | Models | Tools | Workflow | Upgrade | Doctor ]`.
- Render a 7-stage progress gauge and a table listing active checkpoints from `state.json`.

### 2. Extended Doctor Health Checks (`src/commands/doctor.rs`)
- Implement `doctor_check_engram_db()`: Verifies SQLite file existence and read permissions at `~/.engram/engram.db`.
- Implement `doctor_check_codegraph_index()`: Verifies `.codegraph/` presence in project root.
- Implement `doctor_check_rtk_path()`: Probes PATH for `rtk` binary.

### 3. Sync Watcher (`src/commands/sync.rs`)
- Add `--watch` flag to `ce-ai sync` command line options.
