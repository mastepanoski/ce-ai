# Implementation Plan: Kimi Adapter Audit Refinements

Address all 3 audit findings from the Kimi Code CLI Native Harness Adapter audit:
1. **Fix Project Rule Location**: Change `init_prj` and `deinit_prj` for Kimi Code CLI from `.kimi-code/rules/compound-engineering.md` (legacy `kimi-cli` format) to `.kimi-code/AGENTS.md` (official Kimi Code CLI instruction file format).
2. **Generic JSON Cleanup**: Remove legacy reference to Kimi from module doc comments in `src/harness/generic_json.rs`.
3. **Helper Decoupling**: Rename or export managed rule block update/strip helpers (`update_managed_rule_md`, `strip_managed_block`) cleanly in `src/harness/grok.rs` / `src/commands/init_prj.rs` to avoid awkward cross-adapter coupling.

## User Review Required
> [!NOTE]
> All changes maintain 100% backward compatibility while aligning with official Kimi Code CLI specifications (`.kimi-code/AGENTS.md`).

## Proposed Changes

### 1. OpenSpec Contract (`openspec/changes/kimi-adapter-audit-refinements/`)
- Create `proposal.md`, `exploration.md`, `design.md`, `spec.md`, and `tasks.md`.
- Amend R3 in `openspec/changes/kimi-native-harness-adapter/spec.md`.

### 2. `src/harness/mod.rs`
- Export `update_managed_rule_md` and `strip_managed_rule_block` as neutral helper functions for managed markdown rule updates.

### 3. `src/commands/init_prj.rs` & `src/commands/deinit_prj.rs`
- Update Kimi project rule adoption to target `.kimi-code/AGENTS.md` if `.kimi-code` directory exists.
- Use `crate::harness::update_managed_rule_md` and `crate::harness::strip_managed_rule_block` across adapters instead of `grok::update_grok_rule_md`.
- In `deinit_prj.rs`, clean up `.kimi-code/AGENTS.md` managed block, deleting the file if empty. Also clean up legacy `.kimi-code/rules/compound-engineering.md` if present.

### 4. `src/harness/generic_json.rs`
- Clean up module doc comment removing legacy Kimi and Antigravity references.

### 5. `tests/cli.rs`
- Update `init_prj_kimi_writes_and_deinits_agents_md` to verify `.kimi-code/AGENTS.md` adoption and deinit lifecycle.

## Verification Plan

### Automated Tests
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `make e2e`
