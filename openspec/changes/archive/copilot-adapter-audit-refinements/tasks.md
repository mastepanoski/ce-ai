# Task Breakdown: Copilot Adapter Audit Refinements

- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] Implementation:
  - [x] Update `register_copilot_mcp_server` env map handling in `src/harness/copilot.rs`
  - [x] Update skills directory removal warning emission in `src/commands/uninstall.rs`
  - [x] Amend `openspec/changes/copilot-native-harness-adapter/design.md` and `spec.md` to document `COPILOT_CONFIG_DIR`
  - [x] Add unit test `replaces_env_map_cleanly_on_re_registration` in `src/harness/copilot.rs`
  - [x] Add unit/CLI test verifying skills cleanup warning emission in `src/commands/uninstall.rs`
- [x] Quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.13.1`, release
