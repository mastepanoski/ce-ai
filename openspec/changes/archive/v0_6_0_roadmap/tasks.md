# OpenSpec Tasks: Release v0.6.0 Implementation

- [x] **Unit 1: TUI Workflow Dashboard (`src/tui.rs`)**
  - [x] Add `Workflow` tab to TUI tab selector.
  - [x] Render 7-stage flywheel gauge and checkpoint history table.
- [x] **Unit 2: Extended Doctor Health Checks (`src/commands/doctor.rs`)**
  - [x] Implement Engram SQLite DB, CodeGraph index, and RTK binary health checks.
- [x] **Unit 3: Real-Time Sync Watcher (`src/commands/sync.rs`)**
  - [x] Add `--watch` flag to `ce-ai sync`.
- [x] **Unit 4: Integration Tests (`tests/cli.rs`)**
  - [x] Write CLI integration tests for TUI workflow tab data loading and extended doctor diagnostics.
