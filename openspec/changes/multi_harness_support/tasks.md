# OpenSpec Tasks Checklist (TDD Red-Green-Refactor)

**Change Identifier:** `multi_harness_support`  

---

## Task Checklist

### Phase 1: Harness Domain Model & CLI Expansion
- [x] **Task 1.1**: Define `HarnessKind` enum and `HarnessAdapter` trait in `src/harness/mod.rs`.
  - *Verification*: `cargo test harness::tests::enum_parsing_and_resolution`
- [x] **Task 1.2**: Update CLI Clap subcommand parser for `--harness` and `--all` flags in `src/main.rs`.
  - *Verification*: `cargo test tests::cli::install_unknown_harness_exits_usage_code`

### Phase 2: Native Harness Adapters Implementation
- [x] **Task 2.1**: Refactor OpenCode harness to implement `HarnessAdapter` trait in `src/harness/opencode.rs`.
  - *Verification*: `cargo test harness::opencode::tests::roundtrip`
- [x] **Task 2.2**: Implement Claude Code adapter in `src/harness/claude.rs`.
  - *Verification*: `cargo test harness::claude::tests::merges_claude_json`
- [x] **Task 2.3**: Implement Pi harness adapter in `src/harness/pi.rs`.
  - *Verification*: `cargo test harness::pi::tests::merges_pi_config`
- [x] **Task 2.4**: Implement Cursor & Copilot Markdown block adapters in `src/harness/cursor.rs` & `copilot.rs`.
  - *Verification*: `cargo test harness::cursor::tests::managed_block_injection_and_stripping`
- [x] **Task 2.5**: Implement Generic JSON adapter for Codex, Grok, Kimi, AGY, DeepSeek in `src/harness/generic_json.rs`.
  - *Verification*: `cargo test harness::generic::tests::roundtrip`
  - *Reality note (v1.19.1)*: `generic_json.rs` only ever implemented the Custom kind and was removed in v1.19.0 (single path contract). Codex/Grok/Kimi/AGY ship native adapters from the audit series (#199–#203); DeepSeek was de-scoped (Issue #180). Live verification: `cargo test harness::custom::tests::custom_adapter_default_paths_use_single_contract`.
- [x] **Task 2.6**: Implement Custom harness fallback mode (`--harness custom`) in `src/harness/custom.rs`.
  - *Verification*: `cargo test harness::custom::tests::custom_flags_registration`
  - *Reality note (v1.19.0)*: R4 landed for real in PR #205; the cited test never existed. Live verification: `cargo test --test cli install_custom`.

### Phase 3: Model Role Translation & Multi-Harness Sync
- [x] **Task 3.1**: Implement multi-harness model translation engine in `src/commands/models.rs`.
  - *Verification*: `cargo test commands::models::tests::syncs_across_all_active_harnesses`
- [x] **Task 3.2**: Implement `--all` host harness auto-detection.
  - *Verification*: `cargo test harness::tests::auto_detects_installed_harnesses`

### Phase 4: Containerized E2E Gate & Verification
- [x] **Task 4.1**: Expand Docker E2E gate runner (`e2e_runner.sh`) to test multi-harness installation, sync, model setting, and uninstallation.
  - *Verification*: `make e2e`
- [x] **Task 4.2**: Verify 100% green cross-platform CI matrix across Linux, macOS, and Windows.
  - *Verification*: `./.githooks/pre-commit` & GitHub Actions CI.
