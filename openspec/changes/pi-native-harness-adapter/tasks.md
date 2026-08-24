# Task Breakdown: Pi Native Harness Adapter

- [x] Implement `PiAdapter` in `src/harness/pi.rs` targeting `~/.pi/agent/` and `$PI_CODING_AGENT_DIR`
- [x] Update `harness_dir`, `config_path`, `is_installed_on_host`, and `is_ce_installed` for `HarnessKind::Pi` in `src/harness/mod.rs`
- [x] Update `src/commands/install.rs` for `HarnessKind::Pi` skills installation
- [x] Update `src/commands/uninstall.rs` for `HarnessKind::Pi` skills cleanup
- [x] Update `src/commands/tools.rs` for `pi` MCP unsupported handling
- [x] Update `src/commands/init_prj.rs` and `deinit_prj.rs` for `.pi/AGENTS.md`
- [x] Add unit and CLI integration tests in `src/harness/pi.rs` and `tests/cli.rs`
- [x] Run `ce-doc-review` panel
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound` at `docs/solutions/architecture/pi-native-harness-adapter.md`
- [x] Create branch `feat/pi-native-harness-adapter`, commit, push, PR, merge, release minor `v1.16.0`
