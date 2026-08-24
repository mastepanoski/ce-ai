# Proposal: Kimi Adapter Audit Refinements

## Problem Statement
Audit findings on Issue #178 (Kimi Code CLI Native Harness Adapter) identified three refinements:
1. `ce-ai init-prj` writes project rules for Kimi to `.kimi-code/rules/compound-engineering.md`. However, Kimi Code CLI officially loads instructions from `AGENTS.md` (root), `.kimi-code/AGENTS.md`, and `$KIMI_CODE_HOME/AGENTS.md`. The `.kimi-code/rules/` path was legacy `kimi-cli` and is ignored by Kimi Code CLI.
2. `src/harness/generic_json.rs` contains outdated doc comments mentioning Kimi.
3. Reusing `grok::update_grok_rule_md` across non-Grok adapters creates awkward cross-adapter coupling.

## In-Scope
- Update Kimi project rule adoption in `init_prj.rs` and `deinit_prj.rs` to target `.kimi-code/AGENTS.md`.
- Clean up doc comments in `src/harness/generic_json.rs`.
- Extract generic managed rule block update and strip helpers (`update_managed_rule_md`, `strip_managed_rule_block`) into `src/harness/mod.rs` to eliminate cross-adapter coupling on `grok::update_grok_rule_md`.
- Update integration tests in `tests/cli.rs`.
- Amend R3 in `openspec/changes/kimi-native-harness-adapter/spec.md`.

## Out-of-Scope
- Modifications to `mcpServers` JSON structure or skills directory path.
