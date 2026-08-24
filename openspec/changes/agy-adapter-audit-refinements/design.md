# Design: Google Antigravity (AGY) Adapter Audit Refinements

## Environment Variable Extensions
`ce-ai` provides two environment variable extensions for custom directory relocation when targeting Google Antigravity:
1. `ANTIGRAVITY_CONFIG_DIR`: Highest precedence override for `harness_dir`.
2. `GEMINI_HOME`: Secondary precedence override for `harness_dir`.
3. Default: `<home_dir>/.gemini`.

Note: `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` are custom `ce-ai` extensions introduced to support hermetic test runners and custom user deployments.

## Project Rules Architecture
- **Canonical Instruction File**: `<project_dir>/GEMINI.md`
- **Derived Stub File**: `<project_dir>/.agents/rules/compound-engineering.md` (adopted when `.agents/` directory pre-exists).

## Server URL Collision Policy
When registering a managed MCP server (`codegraph` or `engram`) via `register_agy_mcp_server`, if an existing entry under that key contains a `serverUrl` field, `server_url` is explicitly set to `None` to convert the entry cleanly to a stdio command definition (`command`, `args`, `env`). Non-colliding remote MCP servers (servers with distinct names) are explicitly preserved unchanged.

## HarnessAdapter Trait Evolution
The `HarnessAdapter` trait evolved cleanly across adapters to expose zero-argument relative path methods:
- `fn canonical_instruction_file(&self) -> PathBuf`: Returns `PathBuf::from("GEMINI.md")`.
- `fn derived_stub_files(&self) -> Vec<PathBuf>`: Returns `vec![PathBuf::from(".agents").join("rules").join("compound-engineering.md")]`.
