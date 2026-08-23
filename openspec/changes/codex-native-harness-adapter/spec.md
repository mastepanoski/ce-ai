# Specification: Codex Native Harness Adapter

## Acceptance Criteria

### R1: Harness Home & Config Path Resolution
- WHEN `harness_dir(HarnessKind::Codex)` is called THEN it MUST return `$CODEX_CONFIG_DIR` if set, otherwise `$HOME/.codex`.
- WHEN `default_config_path` is called THEN it MUST return `$CODEX_CONFIG_DIR/config.toml` if set, otherwise `$HOME/.codex/config.toml`.
- WHEN `is_installed_on_host` is called for `HarnessKind::Codex` THEN it MUST check if the harness directory or `config.toml` exists.
- WHEN `is_ce_installed` is called for `HarnessKind::Codex` THEN it MUST check if `config.toml` contains `[mcp_servers]` sidecars or if `skills/` exists under the harness directory.

### R2: Native TOML MCP Server Configuration
- WHEN `ce-ai install --harness codex` or `ce-ai tools install <tool>` runs THEN sidecar MCP servers MUST be registered under `[mcp_servers.<name>]` in TOML format (`command`, `args`, `env`).
- WHEN writing `~/.codex/config.toml` THEN `ce-ai` MUST NOT write OpenCode keys (`plugin`, `skills.paths`).
- WHEN updating `~/.codex/config.toml` THEN `ce-ai` MUST preserve all unmanaged top-level TOML keys and user `[mcp_servers]` tables.

### R3: Project Rule Adoption
- WHEN `ce-ai init-prj` runs in a Codex-enabled project THEN project rules MUST be updated in:
  1. `./AGENTS.md` if present
  2. `.codex/AGENTS.md` if `.codex/` directory exists
  3. Default to `./AGENTS.md`
- WHEN `ce-ai init-prj` runs THEN it MUST inject or update the demarcated `CE-AI MANAGED BLOCK`.
- WHEN `ce-ai deinit-prj` is run THEN it MUST strip the demarcated `CE-AI MANAGED BLOCK` from the project rule file.

### R4: Uninstallation & Cleanup
- WHEN `uninstall --harness codex` is run THEN `ce-ai` sidecars (`codegraph`, `engram`) MUST be unregistered from `[mcp_servers]`.
- WHEN `[mcp_servers]` becomes empty THEN `config.toml` MUST NOT be deleted to preserve user configuration and authentication credentials.

### R5: Skills Directory & Health Inspection
- WHEN `install` or `sync` runs for Codex harness THEN managed skills MUST be copied to `~/.codex/skills/<name>/SKILL.md`.
- WHEN `ce-ai doctor` or `ce-ai status` runs THEN Codex sidecars and skills status MUST be correctly reported.
