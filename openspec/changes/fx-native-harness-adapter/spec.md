# Specification: fx Native Harness Adapter

## Requirements

### R1: Harness Directory & Environment Resolution
- `harness_dir(Fx)` MUST evaluate `$FX_HOME` if set and non-empty.
- If `$FX_HOME` is unset or empty, `harness_dir(Fx)` MUST return `<home_dir>/.fx`.
- `is_installed_on_host(Fx)` MUST return `true` if `<harness_dir>` exists or `<home_dir>/.fx` exists.

### R2: Native MCP Schema & Array Command Formatting
- `register_fx_mcp_server` MUST write MCP servers under root key `mcp` in `<harness_dir>/mcp.json`, setting `type` to `"local"` for stdio command servers.
- `command` field MUST be formatted as an array `["<binary>", "<arg1>", "<arg2>", ...]`.
- `environment` field MUST record environment variables as a JSON object map.
- User entries and extra fields in `mcp.json` MUST be preserved verbatim.

### R3: Skills Asset Management
- `ce-ai install fx` MUST copy managed skills to `<harness_dir>/skills/`.
- `is_ce_installed(Fx)` MUST return `true` if `<harness_dir>/mcp.json` or `<harness_dir>/skills/` exists.

### R4: Project Rule Adoption
- `ce-ai init-prj` MUST adopt `AGENTS.md` at the project root by default, and MUST additionally write managed rules to `.fx/AGENTS.md` when `.fx/` directory pre-exists in the project root.
- `ce-ai deinit-prj` MUST strip managed blocks from `AGENTS.md` and `.fx/AGENTS.md`, removing `.fx/AGENTS.md` if empty.

### R5: Clean Lifecycle & Uninstall
- `ce-ai uninstall fx` MUST remove `codegraph`, `engram`, `context7`, `rtk` entries from `mcp.json`, removing `mcp.json` if empty, clean up `<harness_dir>/skills/`, and remove `fx` from `state.json`.
