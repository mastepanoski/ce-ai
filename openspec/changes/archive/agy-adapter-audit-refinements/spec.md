# Specification: Google Antigravity (AGY) Adapter Audit Refinements

## Requirements

### R1: Document Environment Variable Extension Conventions
WHEN inspecting Google Antigravity adapter configuration,
THEN `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` SHALL be documented in OpenSpec design as `ce-ai` extension conventions for custom directory relocation.

### R2: Document Project Rules Locations
WHEN adopting or de-adopting a project for Google Antigravity,
THEN `GEMINI.md` SHALL serve as canonical instruction file and `.agents/rules/compound-engineering.md` SHALL serve as derived stub file.

### R3: Server URL Collision Policy
WHEN `register_agy_mcp_server` updates an existing MCP server entry containing `serverUrl`,
THEN `server_url` SHALL be set to `None` and stdio `command`, `args`, `env` SHALL be written.

### R4: HarnessAdapter Zero-Argument Trait Evolution
WHEN resolving instruction files for Google Antigravity through `HarnessAdapter`,
THEN `canonical_instruction_file()` SHALL return `PathBuf::from("GEMINI.md")` and `derived_stub_files()` SHALL return `vec![PathBuf::from(".agents/rules/compound-engineering.md")]`.
