# OpenSpec Tasks: Release v0.6.0 Implementation

- [ ] **Unit 1: TUI Workflow Dashboard (`src/tui.rs`)**
  - [ ] Add `Workflow` tab to TUI tab selector.
  - [ ] Render 7-stage flywheel gauge and checkpoint history table.
- [ ] **Unit 2: Extended Doctor Health Checks (`src/commands/doctor.rs`)**
  - [ ] Implement Engram SQLite DB, CodeGraph index, and RTK binary health checks.
- [ ] **Unit 3: Real-Time Sync Watcher (`src/commands/sync.rs`)**
  - [ ] Add `--watch` flag to `ce-ai sync`.
- [ ] **Unit 4: Integration Tests (`tests/cli.rs`)**
  - [ ] Write CLI integration tests for TUI workflow tab data loading and extended doctor diagnostics.
