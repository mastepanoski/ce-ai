# Task Breakdown: Codex Adapter Audit Refinements

- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] Implementation:
  - [x] Replace `CODEX_CONFIG_DIR` with `CODEX_HOME` in `src/harness/mod.rs` and `src/harness/codex.rs`
  - [x] Update `register_codex_mcp_server` env map replacement in `src/harness/codex.rs`
  - [x] Remove `HarnessKind::Codex` from `src/harness/generic_json.rs`
  - [x] Amend `openspec/changes/codex-native-harness-adapter/spec.md` (R1 and R3) to document `CODEX_HOME` and `.codex/AGENTS.md` adoption
  - [x] Update unit and CLI tests in `tests/cli.rs` and `src/harness/codex.rs`
- [x] Quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for green CI, merge, tag `v1.12.1`, release
