# Specification: Pi Native Harness Adapter

## Requirements

### R1: Harness Directory & Environment Resolution
- `harness_dir(Pi)` MUST evaluate `$PI_CODING_AGENT_DIR` if set and non-empty.
- If `$PI_CODING_AGENT_DIR` is unset or empty, `harness_dir(Pi)` MUST return `<home_dir>/.pi/agent`.
- `is_installed_on_host(Pi)` MUST return `true` if `<harness_dir>` exists or `<home_dir>/.pi` exists.

### R2: Skills Asset Management
- `ce-ai install pi` MUST copy managed skills to `<harness_dir>/skills/`.
- `ce-ai install pi` MUST NOT write fictional `config.json` or OpenCode JSON files.

### R3: Project Rule Adoption
- `ce-ai init-prj` MUST adopt `AGENTS.md` at the project root by default, and MUST additionally write managed rules to `.pi/AGENTS.md` when `.pi/` directory pre-exists in the project root.
- `ce-ai deinit-prj` MUST strip managed blocks from `AGENTS.md` and `.pi/AGENTS.md`, removing `.pi/AGENTS.md` if empty.

### R4: Companion Tools MCP Handling
- `ce-ai tools install` MUST report MCP as unsupported for `pi` targets with an informative notice and gracefully skip MCP configuration generation without failing multi-harness tool installation.

### R5: Clean Lifecycle & Uninstall
- `is_ce_installed(Pi)` MUST return `true` if `<harness_dir>/skills/` exists and is non-empty.
- `ce-ai uninstall pi` MUST remove `<harness_dir>/skills/` and clean up `state.json` entries cleanly.
