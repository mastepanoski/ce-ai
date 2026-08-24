# Specification: Antigravity (AGY) Adapter Audit Refinements

## Requirements

### R1: Environment Variable Extensions Documentation
- `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` MUST be documented as `ce-ai` extension conventions for custom harness directory overrides.

### R2: Name Collision Policy Specification
- `register_agy_mcp_server` MUST reset `serverUrl` to `None` when registering a local stdio command server over an existing remote server entry sharing the same name.
- Unmanaged remote server entries with distinct names MUST be preserved with their `serverUrl` and `headers` fields intact.

### R3: Project Rule Adoption Specification
- `canonical_instruction_file` MUST designate `GEMINI.md` as the canonical project instruction file.
- `derived_stub_files` MUST include `.agents/rules/compound-engineering.md` as a derived stub when `.agents/` pre-exists.
