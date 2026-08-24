# Design: Kimi Adapter Audit Refinements

## 1. Project Adoption Logic (`src/commands/init_prj.rs`)
- When `.kimi-code/` directory exists in the target project, `ce-ai init-prj` writes the managed block to `.kimi-code/AGENTS.md`.

## 2. Project De-adoption Logic (`src/commands/deinit_prj.rs`)
- `ce-ai deinit-prj` strips `CE-AI MANAGED BLOCK` from `.kimi-code/AGENTS.md`. If the file becomes empty after stripping, it is deleted.
- Also strips managed block from legacy `.kimi-code/rules/compound-engineering.md` if present.

## 3. Generic JSON Module Cleanup (`src/harness/generic_json.rs`)
- Update header doc comment to remove references to Kimi and Antigravity.

## 4. Shared Managed Rule Block Helpers (`src/harness/mod.rs`)
- Export `pub fn update_managed_rule_md(path: &Path, body: &str) -> Result<(), CeError>` and `pub fn strip_managed_rule_block(content: &str) -> String` from `src/harness/mod.rs` (delegating to or unifying rule update logic) to avoid non-Grok adapters calling `grok::update_grok_rule_md`.
