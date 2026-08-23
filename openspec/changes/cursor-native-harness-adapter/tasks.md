# Task Breakdown: Cursor Native Harness Adapter

- [x] Create brainstorm doc `docs/brainstorms/2026-08-23-cursor-native-harness-adapter-requirements.md`
- [x] Create plan doc `docs/plans/2026-08-23-cursor-native-harness-adapter-plan.md`
- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract
- [ ] TDD Implementation:
  - [ ] Update `CursorAdapter::default_config_path` in `src/harness/cursor.rs` to return `home.join(".cursor").join("mcp.json")`
  - [ ] Implement `CursorMcpConfig`, `CursorMcpServer`, `CursorRuleFrontmatter` in `src/harness/cursor.rs`
  - [ ] Implement `register_cursor_mcp_server`, `unregister_cursor_mcp_server`, `update_cursor_rule_mdc` with unit tests in `src/harness/cursor.rs`
  - [ ] Wire Cursor native install in `src/commands/install.rs` and `src/commands/tools.rs`
  - [ ] Wire Cursor workspace rule in `src/commands/init_prj.rs` and drift sync in `src/commands/sync.rs`
  - [ ] Wire Cursor native uninstall in `src/commands/uninstall.rs`
  - [ ] Add CLI integration tests verifying `mcpServers` stdio schema, zero OpenCode keys (`plugin`, `skills.paths`), `.cursor/rules/*.mdc` format, and clean uninstall in `tests/cli.rs`
- [ ] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [ ] Run `ce-code-review` panel
- [ ] Document solution via `ce-compound`
- [ ] Ship PR, wait for 100% green CI matrix, merge, tag `v1.9.0`, release, close Issue #173
