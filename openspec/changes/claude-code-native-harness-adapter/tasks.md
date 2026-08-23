# Tasks: Claude Code Native Harness Adapter (Issue #174)

- [ ] Implement `src/harness/claude.rs` with `ClaudeAdapter`, `ClaudeMcpConfig`, `ClaudeMcpServer`, `register_claude_mcp_server`, `unregister_claude_mcp_server`, and `update_claude_md` <!-- id: 0 -->
- [ ] Update `ClaudeAdapter::default_config_path` and `HarnessKind::Claude.harness_dir` in `src/harness/mod.rs` <!-- id: 1 -->
- [ ] Wire Claude native installation and skills directory (`~/.claude/skills/`) in `src/commands/install.rs` <!-- id: 2 -->
- [ ] Wire Claude companion tool registration in `src/commands/tools.rs` <!-- id: 3 -->
- [ ] Wire Claude project adoption (`init-prj`) and de-adoption (`deinit-prj`) in `src/commands/init_prj.rs` and `src/commands/deinit_prj.rs` <!-- id: 4 -->
- [ ] Wire Claude drift reconciliation in `src/commands/sync.rs` <!-- id: 5 -->
- [ ] Wire Claude sidecar unregistration in `src/commands/uninstall.rs` <!-- id: 6 -->
- [ ] Update backup harness tagging in `src/state/backups.rs` <!-- id: 7 -->
- [ ] Add unit tests in `src/harness/claude.rs` and CLI integration tests in `tests/cli.rs` <!-- id: 8 -->
