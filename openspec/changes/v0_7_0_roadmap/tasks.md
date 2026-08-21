# OpenSpec Tasks: Release v0.7.0 Implementation Plan

- [ ] **Unit 1: Workspace Overrides Merging Engine (`src/state/state.rs`)**
  - [ ] Implement `State::load_with_workspace_overrides(global_path, workspace_root)`.
  - [ ] Implement `merge_overrides` for model assignments and profile overrides.
  - [ ] Add unit tests in `src/state/state.rs` verifying local override precedence.

- [ ] **Unit 2: Multi-Harness Uninstall Extension (`src/harness/mod.rs` & `src/commands/uninstall.rs`)**
  - [ ] Extend `HarnessAdapter` trait with `uninstall(&self, ctx: &Context, all: bool) -> Result<(), CeError>`.
  - [ ] Implement `uninstall` logic across all harness adapters (`opencode`, `claude`, `cursor`, `copilot`, `pi`, `antigravity`, etc.).
  - [ ] Add `--harness <name|all>`, `--all`, and `--yes` flags to `UninstallArgs` in `src/main.rs` and `src/commands/uninstall.rs`.

- [ ] **Unit 3: CLI Integration Tests (`tests/cli.rs`)**
  - [ ] Add integration test `workspace_overrides_precedence_test` in `tests/cli.rs`.
  - [ ] Add integration test `uninstall_harness_all_with_yes_flag_test` in `tests/cli.rs`.
