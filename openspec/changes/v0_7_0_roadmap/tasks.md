# OpenSpec Tasks: Release v0.7.0 Implementation Plan

- [x] **Unit 1: Workspace Overrides Merging Engine (`src/state/state.rs`)**
  - [x] Implement `State::load_with_workspace_overrides(global_path, workspace_root)`.
  - [x] Implement `merge_overrides` for model assignments and profile overrides.
  - [x] Add unit tests in `src/state/state.rs` verifying local override precedence.

- [x] **Unit 2: Multi-Harness Uninstall Extension (`src/harness/mod.rs` & `src/commands/uninstall.rs`)**
  - [x] Extend `HarnessAdapter` trait with `uninstall(&self, ctx: &Context, all: bool) -> Result<(), CeError>`.
  - [x] Implement `uninstall` logic across all harness adapters (`opencode`, `claude`, `cursor`, `copilot`, `pi`, `antigravity`, etc.).
  - [x] Add `--harness <name|all>`, `--all`, and `--yes` flags to `UninstallArgs` in `src/main.rs` and `src/commands/uninstall.rs`.

- [x] **Unit 3: CLI Integration Tests (`tests/cli.rs`)**
  - [x] Add integration test `workspace_overrides_precedence_test` in `tests/cli.rs`.
  - [x] Add integration test `uninstall_harness_all_with_yes_flag_test` in `tests/cli.rs`.
