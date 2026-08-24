# Task Breakdown: GitHub Copilot Native Harness Adapter

- [x] Create brainstorm doc `docs/brainstorms/2026-08-23-copilot-native-harness-adapter-requirements.md`
- [x] Create plan doc `docs/plans/2026-08-23-copilot-native-harness-adapter-plan.md`
- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] TDD Implementation:
  - [x] Implement `CopilotAdapter` in `src/harness/copilot.rs`
  - [x] Implement `register_copilot_mcp_server`, `unregister_copilot_mcp_server`, `update_copilot_instructions_md` with unit tests in `src/harness/copilot.rs`
  - [x] Update `is_installed_on_host` and `is_ce_installed` for `HarnessKind::Copilot` in `src/harness/mod.rs`
  - [x] Wire Copilot native install in `src/commands/install.rs` and `src/commands/tools.rs`
  - [x] Wire Copilot project rule in `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, and drift sync in `src/commands/sync.rs`
  - [x] Wire Copilot native uninstall in `src/commands/uninstall.rs`
  - [x] Tag Copilot backups with `copilot-` prefix in `src/state/backups.rs`
  - [x] Update health check and status reporting in `src/commands/doctor.rs` and `src/commands/status.rs`
  - [x] Add CLI integration tests verifying JSON `mcpServers` schema, zero OpenCode keys, `.github/copilot-instructions.md` rule creation, and clean uninstall in `tests/cli.rs`
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.12.0`, release, close Issue #177
