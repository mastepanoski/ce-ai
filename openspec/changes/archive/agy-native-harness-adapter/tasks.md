# Task Breakdown: Google Antigravity (agy) Native Harness Adapter (Issue #179)

- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] Implementation:
  - [x] Implement `src/harness/agy.rs` with `AgyAdapter`, `AgyMcpConfig`, `AgyMcpServer`, `register_agy_mcp_server`, and `unregister_agy_mcp_server`
  - [x] Wire `HarnessKind::Agy` in `src/harness/mod.rs`
  - [x] Wire `Agy` in subcommands (`install.rs`, `tools.rs`, `init_prj.rs`, `deinit_prj.rs`, `sync.rs`, `uninstall.rs`, `backups.rs`)
  - [x] Remove legacy `HarnessKind::Agy` generic mapping from `src/harness/generic_json.rs`
  - [x] Add unit tests in `src/harness/agy.rs` verifying `mcpServers` JSON schema, `serverUrl` preservation, zero OpenCode keys, and `$ANTIGRAVITY_CONFIG_DIR`/`$GEMINI_HOME` lock via `HARNESS_ENV_LOCK`
  - [x] Add integration tests in `tests/cli.rs` verifying install, `GEMINI.md` adoption, and clean uninstall lifecycle
  - [x] Bump SemVer to `v1.15.0` in `Cargo.toml` and `Formula/ce-ai.rb`, and update `CHANGELOG.md`
- [x] Quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.15.0`, release, close Issue #179
