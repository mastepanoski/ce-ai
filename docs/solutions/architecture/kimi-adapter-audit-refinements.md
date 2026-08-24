---
module: harness
tags: [kimi, harness, adapter, agents_md, rules, audit]
problem_type: architectural_refactor
---

# Solution: Kimi Adapter Audit Refinements

## Problem
Audit of Kimi Code CLI Native Harness Adapter identified three issues:
1. `init-prj` wrote project rules to `.kimi-code/rules/compound-engineering.md` (legacy `kimi-cli` path). Official Kimi Code CLI documentation specifies loading instructions from `AGENTS.md` (root), `.kimi-code/AGENTS.md`, and `$KIMI_CODE_HOME/AGENTS.md`. Managed blocks in `rules/` were ignored by Kimi Code CLI.
2. `src/harness/generic_json.rs` contained obsolete module doc comments mentioning Kimi.
3. Cross-adapter coupling: non-Grok adapters (Kimi, Antigravity) were directly invoking `grok::update_grok_rule_md` and `grok::CE_MANAGED_BEGIN`.

## Solution Details
1. **Official Kimi Instruction File**:
   - `src/commands/init_prj.rs`: Target `.kimi-code/AGENTS.md` when `.kimi-code/` directory exists.
   - `src/commands/deinit_prj.rs`: Clean up `.kimi-code/AGENTS.md` managed block, removing the file if empty. Also clean up legacy `.kimi-code/rules/compound-engineering.md` and empty parent `rules/` directory if present.
2. **Neutral Rule Helpers**:
   - Exported `update_managed_rule_md`, `strip_managed_rule_block`, and `CE_MANAGED_BEGIN` from `src/harness/mod.rs` to eliminate direct coupling on `grok.rs`.
3. **Generic JSON Cleanup**:
   - Cleaned up module documentation header in `src/harness/generic_json.rs`.
4. **OpenSpec Alignment**:
   - Amended R3 in `openspec/changes/kimi-native-harness-adapter/spec.md`.

## Verification
- CLI integration tests (`init_prj_kimi_writes_and_deinits_agents_md` verifying `.kimi-code/AGENTS.md` adoption and legacy `rules/` cleanup).
- 100% green test suite (137 unit tests, 73 integration tests).
