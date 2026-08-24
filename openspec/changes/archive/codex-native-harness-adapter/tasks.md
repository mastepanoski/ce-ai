# Task Breakdown: Codex Native Harness Adapter

- [x] Create brainstorm doc `docs/brainstorms/2026-08-23-codex-native-harness-adapter-requirements.md`
- [x] Create plan doc `docs/plans/2026-08-23-codex-native-harness-adapter-plan.md`
- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Add `toml = "0.8"` dependency to `Cargo.toml`
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] TDD Implementation:
  - [x] Implement `CodexAdapter` in `src/harness/codex.rs`
  - [x] Implement `register_codex_mcp_server`, `unregister_codex_mcp_server`, `update_codex_agents_md` with unit tests in `src/harness/codex.rs`
  - [x] Update `is_installed_on_host` and `is_ce_installed` for `HarnessKind::Codex` in `src/harness/mod.rs`
  - [x] Wire Codex native install in `src/commands/install.rs` and `src/commands/tools.rs`
  - [x] Wire Codex project rule in `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, and drift sync in `src/commands/sync.rs`
  - [x] Wire Codex native uninstall in `src/commands/uninstall.rs`
  - [x] Update health check and status reporting in `src/commands/doctor.rs` and `src/commands/status.rs`
  - [x] Add CLI integration tests verifying TOML `[mcp_servers]` schema, zero OpenCode keys, `AGENTS.md` rule creation, and clean uninstall in `tests/cli.rs`
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.11.0`, release, close Issue #175
