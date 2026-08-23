# Task Breakdown: Claude Code Native Harness Adapter

- [x] Create brainstorm doc `docs/brainstorms/2026-08-23-claude-code-native-harness-adapter-requirements.md`
- [x] Create plan doc `docs/plans/2026-08-23-claude-code-native-harness-adapter-plan.md`
- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract
- [x] TDD Implementation:
  - [x] Implement `ClaudeAdapter` and `ClaudeMcpConfig` in `src/harness/claude.rs`
  - [x] Implement `register_claude_mcp_server`, `unregister_claude_mcp_server`, `update_claude_md`, and `strip_managed_block` in `src/harness/claude.rs`
  - [x] Wire Claude native install in `src/commands/install.rs` and `src/commands/tools.rs`
  - [x] Wire Claude project rule in `src/commands/init_prj.rs` and drift sync in `src/commands/sync.rs`
  - [x] Wire Claude native uninstall in `src/commands/uninstall.rs`
  - [x] Add CLI integration tests verifying native `mcpServers` stdio schema, zero OpenCode keys, `CLAUDE.md` rule creation, and clean uninstall in `tests/cli.rs`
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.10.0`, release, close Issue #174
