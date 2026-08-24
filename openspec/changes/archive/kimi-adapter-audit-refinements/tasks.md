# Task Breakdown: Kimi Adapter Audit Refinements

- [x] Update `init_prj.rs` to write `.kimi-code/AGENTS.md` when `.kimi-code` directory exists
- [x] Update `deinit_prj.rs` to clean up `.kimi-code/AGENTS.md` and legacy `.kimi-code/rules/compound-engineering.md`
- [x] Clean up `src/harness/generic_json.rs` module header comment
- [x] Extract neutral rule update helpers `update_managed_rule_md` and `strip_managed_rule_block` into `src/harness/mod.rs`
- [x] Update integration test `init_prj_kimi_writes_and_deinits_agents_md` in `tests/cli.rs`
- [x] Amend R3 in `openspec/changes/kimi-native-harness-adapter/spec.md`
- [x] Run quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Create branch `fix/kimi-adapter-audit-refinements`, commit, push, PR, merge, release patch `v1.15.1`
