# Specification: Kimi Adapter Audit Refinements

> **Note**: Amends Requirement R3 of `openspec/changes/kimi-native-harness-adapter/spec.md` by updating the target project rule adoption file from `.kimi-code/rules/compound-engineering.md` to `.kimi-code/AGENTS.md`.

## Requirements

### R1: Project Rule Adoption File Location
- WHEN running `init-prj` for a project containing `.kimi-code/` THEN `ce-ai` MUST write the `CE-AI MANAGED BLOCK` into `.kimi-code/AGENTS.md`.
- WHEN running `deinit-prj` THEN `ce-ai` MUST strip the `CE-AI MANAGED BLOCK` from `.kimi-code/AGENTS.md` (and legacy `.kimi-code/rules/compound-engineering.md` if present).
- WHEN `deinit-prj` strips the managed block and the target file (`.kimi-code/AGENTS.md` or legacy `.kimi-code/rules/compound-engineering.md`) becomes empty, THEN `ce-ai` MUST delete the empty file.

### R2: Generic JSON Dead Code Elimination
- `src/harness/generic_json.rs` MUST NOT reference Kimi in module documentation.

### R3: Rule Update Helper Decoupling
- Managed rule block update and stripping logic MUST be exposed via neutral helper functions in `src/harness/mod.rs` (`update_managed_rule_md`, `strip_managed_rule_block`), eliminating cross-adapter dependencies on `grok::update_grok_rule_md`.
