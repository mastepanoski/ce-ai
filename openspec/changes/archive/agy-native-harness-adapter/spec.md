# Spec: Google Antigravity (agy) Native Harness Adapter

## Requirements

### R1: Native Directory & Config File Resolution
WHEN `ce-ai` determines paths for `HarnessKind::Agy`
THEN it MUST check environment variable `$ANTIGRAVITY_CONFIG_DIR` first, then `$GEMINI_HOME`, and default to `<home_dir>/.gemini`
AND it MUST locate the native MCP config file at `<harness_dir>/config/mcp_config.json`.

### R2: Native `mcpServers` JSON Schema Compliance
WHEN registering or unregistering MCP servers for Google Antigravity
THEN `ce-ai` MUST write to top-level object `"mcpServers"` in `<harness_dir>/config/mcp_config.json`
AND it MUST map stdio servers with `"command"`, `"args"`, and `"env"`
AND it MUST preserve remote server entries using `"serverUrl"`
AND it MUST preserve pre-existing user-defined servers and top-level JSON keys
AND it MUST NOT leak OpenCode keys (`plugin`, `skills.paths`).

### R3: Skills Directory & Legacy File Management
WHEN `ce-ai install --harness agy` is executed
THEN it MUST populate skills under `<harness_dir>/config/skills/`
AND WHEN `ce-ai uninstall --harness agy` is executed
THEN it MUST clean managed skills under `<harness_dir>/config/skills/` without touching user custom skills
AND it MUST clean any pre-existing legacy `<harness_dir>/antigravity-cli/antigravity.json` config file.

### R4: Project Rule Adoption
WHEN `ce-ai init-prj` is run in a project containing `.agents` directory or adopted for `agy`
THEN `ce-ai` MUST adopt project rules into `.agents/rules/compound-engineering.md` and `GEMINI.md`
AND WHEN `ce-ai deinit-prj` is run
THEN `ce-ai` MUST strip the `CE-AI MANAGED BLOCK` cleanly.
