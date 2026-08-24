# Exploration: Kimi Adapter Audit Refinements

## Technical Investigation
1. **Kimi Code CLI Instruction Discovery**:
   - Official docs for Kimi Code CLI state that user instructions are discovered from:
     - `AGENTS.md` (project root)
     - `.kimi-code/AGENTS.md` (project hidden config dir)
     - `$KIMI_CODE_HOME/AGENTS.md` (global user config dir)
   - `.kimi-code/rules/*.md` is not in the search list. Therefore, managed rule blocks written under `rules/` are invisible to Kimi Code.
2. **Deinit Backward Compatibility**:
   - `deinit-prj` should clean up `.kimi-code/AGENTS.md` managed block, but also clean up legacy `.kimi-code/rules/compound-engineering.md` if existing from earlier versions.
3. **Generic JSON Stale Comments**:
   - `src/harness/generic_json.rs` contained header doc comments referencing Kimi and Antigravity as generic JSON harnesses. Since both have native adapters, those references should be removed.
4. **Helper Decoupling**:
   - `grok::update_grok_rule_md` and `grok::strip_managed_block` were invoked by `init_prj` and `deinit_prj` for non-Grok adapters (Kimi, Antigravity). Moving shared helper functions into `src/harness/mod.rs` (`update_managed_rule_md`, `strip_managed_rule_block`) decouples adapter modules.

## Evaluated Options
- **Option A (Target `.kimi-code/AGENTS.md`)**: Recommended. Aligns directly with Kimi Code CLI specification and existing harness patterns (e.g., Codex using `.codex/AGENTS.md`).
- **Option B (Target root `AGENTS.md` only)**: If `.kimi-code` directory exists, placing `.kimi-code/AGENTS.md` keeps harness-specific instructions neatly scoped.
