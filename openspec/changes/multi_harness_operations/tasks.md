# OpenSpec Tasks: Multi-Harness Operations & TUI Scope

- [ ] **Unit 1: Local Source Guard & Upgrade Refinement (`src/commands/upgrade.rs`)**
  - [ ] Add `force: bool` flag to `upgrade::Args`.
  - [ ] Check installation source in state; skip or prompt before upgrading `local` source.
- [ ] **Unit 2: Multi-Harness Sync (`src/commands/sync.rs`)**
  - [ ] Support `--harness <name>` or `all` in `sync::run`.
- [ ] **Unit 3: TUI Global Target Selector (`src/tui.rs`)**
  - [ ] Add global `selected_harness_target_idx` (with `All Installed` as first option).
  - [ ] Update TUI rendering and key event handlers across all tabs.
- [ ] **Unit 4: CLI Integration Tests (`tests/cli.rs`)**
  - [ ] Add tests for multi-harness sync, upgrade local source protection, and TUI target selection.
