# Task Breakdown: Kimi Code CLI Native Harness Adapter (Issue #178)

- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] Implementation:
  - [x] Implement `src/harness/kimi.rs` with `KimiAdapter`, `KimiMcpConfig`, `KimiMcpServer`, `register_kimi_mcp_server`, and `unregister_kimi_mcp_server`
  - [x] Wire `HarnessKind::Kimi` in `src/harness/mod.rs`
  - [x] Wire `Kimi` in subcommands (`install.rs`, `tools.rs`, `init_prj.rs`, `deinit_prj.rs`, `sync.rs`, `uninstall.rs`, `backups.rs`)
  - [x] Remove legacy `HarnessKind::Kimi` generic mapping from `src/harness/generic_json.rs`
  - [x] Add unit tests in `src/harness/kimi.rs` verifying `mcpServers` JSON schema, zero OpenCode keys, and `$KIMI_CODE_HOME` lock via `HARNESS_ENV_LOCK`
  - [x] Add integration tests in `tests/cli.rs` verifying install, `AGENTS.md` adoption, and clean uninstall lifecycle
  - [x] Bump SemVer to `v1.14.0` in `Cargo.toml` and `Formula/ce-ai.rb`, and update `CHANGELOG.md`
- [x] Quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.14.0`, release, close Issue #178
