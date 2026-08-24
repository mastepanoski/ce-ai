# Task Breakdown: fx Native Harness Adapter

- [x] Implement `FxAdapter`, `FxMcpConfig`, `FxMcpServer`, `register_fx_mcp_server`, and `unregister_fx_mcp_server` in `src/harness/fx.rs`
- [x] Update `harness_dir`, `config_path`, `is_installed_on_host`, and `is_ce_installed` for `HarnessKind::Fx` in `src/harness/mod.rs`
- [x] Remove `Fx` from `src/harness/generic_json.rs`
- [x] Update `src/commands/install.rs` for `HarnessKind::Fx`
- [x] Update `src/commands/uninstall.rs` for `HarnessKind::Fx`
- [x] Update `src/commands/tools.rs` for `HarnessKind::Fx`
- [x] Update `src/commands/init_prj.rs` and `deinit_prj.rs` for `.fx/AGENTS.md`
- [x] Add unit and CLI integration tests in `src/harness/fx.rs` and `tests/cli.rs`
- [x] Run `ce-doc-review` panel
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`, `make e2e`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound` at `docs/solutions/architecture/fx-native-harness-adapter.md`
- [x] Create branch `feat/fx-native-harness-adapter`, commit, push, PR, merge, release minor `v1.17.0`
