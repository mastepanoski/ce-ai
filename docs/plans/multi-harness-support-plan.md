# Implementation Plan: Multi-Harness Support for `ce-ai`

**Origin Document:** [`docs/brainstorms/multi-harness-support-requirements.md`](../brainstorms/multi-harness-support-requirements.md)  
**OpenSpec Spec:** [`openspec/changes/multi_harness_support/spec.md`](../../openspec/changes/multi_harness_support/spec.md)  
**Target Feature:** Issue #1 & Issue #4 — Multi-Harness Support (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`)  
**Date:** 2026-08-21  

---

## 🎯 Problem Frame & Architecture Overview

`ce-ai` currently manages OpenCode plugin configuration. To support multi-harness development, `ce-ai` must provide a modular **Harness Adapter System** capable of:
1. Parsing and validating 12 harness identifiers (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`) and `--all` host harness probing.
2. Merging JSON configs without clobbering unmanaged keys (`opencode`, `claude`, `pi`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`).
3. Ingesting demarcated comment blocks in Markdown rule files (`cursor`, `copilot`).
4. Accepting custom harness directory flags (`--plugins-dir`, `--skills-dir`, `--rules-file`) with `inquire` interactive prompts for `--harness custom`.
5. Syncing central model assignments (`ce-ai models set`) across all active host harnesses.

---

## 📦 Implementation Units

### Unit 1 (U1): `HarnessKind` Enum & `HarnessAdapter` Trait Abstraction
- **Goal**: Define `HarnessKind` enum, `HarnessAdapter` trait, and CLI `--harness` flag parsing.
- **Files**:
  - `src/harness/mod.rs` (Create core trait & registry)
  - `src/main.rs` (Update Clap parser for `--harness` and `--all`)
  - `src/error.rs` (Ensure exit code mapping: Usage=2, State=3, IO=4, Network=5, Verification=6)
- **Approach**:
  - Implement `HarnessKind` with `FromStr` and `Display` traits.
  - Define `HarnessAdapter` trait methods: `name()`, `default_config_path()`, `install()`, `sync()`, `set_model()`, `uninstall()`.
- **Test Scenarios**:
  - `cargo test harness::tests::enum_parsing_and_resolution`
  - `cargo test tests::cli::install_unknown_harness_exits_usage_code`

### Unit 2 (U2): JSON Harness Adapters (OpenCode, Claude, Pi, Codex, Grok, Kimi, AGY, DeepSeek, FX)
- **Goal**: Implement JSON AST merging without clobbering unmanaged keys and using `write_atomic`.
- **Files**:
  - `src/harness/opencode.rs` (Refactor existing OpenCode logic into adapter)
  - `src/harness/claude.rs` (Claude Code JSON adapter for `~/.claude.json`)
  - `src/harness/pi.rs` (Pi JSON adapter for `~/.pi/config.json`)
  - `src/harness/generic_json.rs` (Generic adapter for Codex, Grok, Kimi, AGY, DeepSeek, FX)
- **Approach**:
  - Parse existing JSON with `serde_json::Value`.
  - Mutate target arrays/keys (`plugins`, `skills`).
  - Write back using `crate::state::write_atomic`.
- **Test Scenarios**:
  - `cargo test harness::claude::tests::merges_claude_json_preserving_user_keys`
  - `cargo test harness::pi::tests::merges_pi_config_and_skills_dir`
  - `cargo test harness::generic::tests::roundtrip_atomic_write`

### Unit 3 (U3): Markdown Instruction Block Adapters (Cursor & Copilot)
- **Goal**: Demarcate and manage instruction blocks in `.cursorrules` and `.github/copilot-instructions.md`.
- **Files**:
  - `src/harness/cursor.rs` (`.cursorrules` / `.cursor/rules/`)
  - `src/harness/copilot.rs` (`.github/copilot-instructions.md`)
- **Approach**:
  - Search for `<!-- CE-AI MANAGED BLOCK BEGIN -->` and `<!-- CE-AI MANAGED BLOCK END -->`.
  - If missing, append block to file.
  - If present, replace block content while preserving surrounding text.
  - On uninstall, strip block cleanly.
- **Test Scenarios**:
  - `cargo test harness::cursor::tests::injects_and_updates_managed_block`
  - `cargo test harness::copilot::tests::uninstalls_stripping_block_cleanly`

### Unit 4 (U4): Custom Harness Fallback Mode (`--harness custom`)
- **Goal**: Support `--plugins-dir`, `--skills-dir`, `--rules-file` CLI flags with interactive `inquire` fallback.
- **Files**:
  - `src/harness/custom.rs`
  - `src/commands/install.rs`
- **Approach**:
  - Read custom flags from `Context`.
  - If missing and TTY active, launch interactive `inquire::Text` prompts.
  - Register custom paths in `~/.ce-ai/state.json`.
- **Test Scenarios**:
  - `cargo test harness::custom::tests::validates_and_persists_custom_dirs`

### Unit 5 (U5): Multi-Harness Model Role Synchronization Engine
- **Goal**: Sync model role assignments (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, `ce-debug`) across all active harnesses.
- **Files**:
  - `src/commands/models.rs`
  - `src/state/state.rs`
- **Approach**:
  - Store model slot assignments in `state.json`.
  - Iterate through all active adapters and invoke `adapter.set_model(slot, model)`.
- **Test Scenarios**:
  - `cargo test commands::models::tests::syncs_models_across_all_installed_harnesses`

### Unit 6 (U6): E2E Integration Suite & Docker Container Validation
- **Goal**: Validate complete multi-harness lifecycle in containerized Docker test runner.
- **Files**:
  - `e2e_runner.sh`
  - `tests/e2e.rs`
- **Approach**:
  - Expand `e2e_runner.sh` to execute `ce-ai install --harness opencode`, `claude`, `cursor`, `custom`.
  - Verify restore on `ce-ai uninstall`.
- **Test Scenarios**:
  - `make e2e` (`cargo test --test e2e -- --ignored`)

---

## 🔒 Security & Governance Compliance

- **ISO/IEC 27001 & 27002**: Cryptographic SHA256 file manifest tracking, `write_atomic` atomic overwrites, secret scanning in pre-commit hook.
- **ISO/IEC 42001 & NIST AI RMF 1.0**: Model role scoping (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, `ce-debug`).
- **Definition of Done (DoD)**: 100% test pass rate, zero Clippy warnings, SemVer update in `Cargo.toml`, and entry in `CHANGELOG.md`.
