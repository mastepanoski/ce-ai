# OpenSpec Tasks: Multi-Harness Operations & TUI Scope

- [x] **Unit 1: Local Source Guard & Upgrade Refinement (`src/commands/upgrade.rs`)**
  - [x] Add `force: bool` flag to `upgrade::Args`.
  - [x] Check installation source in state; skip or prompt before upgrading `local` source.
- [x] **Unit 2: Multi-Harness Sync (`src/commands/sync.rs`)**
  - [x] Support `--harness <name>` or `all` in `sync::run`.
- [x] **Unit 3: TUI Global Target Selector (`src/tui.rs`)**
  - [x] Add global `selected_harness_target_idx` (with `All Installed` as first option).
  - [x] Update TUI rendering and key event handlers across all tabs.
- [x] **Unit 4: CLI Integration Tests (`tests/cli.rs`)**
  - [x] Add tests for multi-harness sync, upgrade local source protection, and TUI target selection.
