# Specification: Grok Native Harness Adapter

## Acceptance Criteria

### R1: Harness Home & Config Path Resolution
- WHEN `harness_dir(HarnessKind::Grok)` is called THEN it MUST return `$GROK_HOME` if set, otherwise `$HOME/.grok`.
- WHEN `default_config_path` is called THEN it MUST return `$GROK_HOME/config.toml` if set, otherwise `$HOME/.grok/config.toml`.
- WHEN `is_installed_on_host` is called for `HarnessKind::Grok` THEN it MUST check if `$GROK_HOME` exists or if `$HOME/.grok` exists.
- WHEN `is_ce_installed` is called for `HarnessKind::Grok` THEN it MUST check if `config.toml` contains `[mcp_servers]` sidecars or if `skills/` exists under the harness directory.

### R2: Native TOML MCP Server Configuration
- WHEN `ce-ai install --harness grok` or `ce-ai tools install <tool>` runs THEN sidecar MCP servers MUST be registered under `[mcp_servers.<name>]` in TOML format (`command`, `args`, `env`).
- WHEN writing `~/.grok/config.toml` THEN `ce-ai` MUST NOT write OpenCode keys (`plugin`, `skills.paths`).
- WHEN updating `~/.grok/config.toml` THEN `ce-ai` MUST preserve all unmanaged top-level TOML keys and user `[mcp_servers]` tables.
- WHEN backups are created for Grok THEN `backups.rs` MUST format snapshot file names with the `grok-` prefix.

### R3: Project Rule Adoption
- WHEN `ce-ai init-prj` runs in a Grok-enabled project THEN project rules MUST be updated in `.grok/rules/compound-engineering.md`.
- WHEN `ce-ai init-prj` runs THEN it MUST inject or update the demarcated `CE-AI MANAGED BLOCK`.
- WHEN `ce-ai deinit-prj` is run THEN it MUST strip the demarcated `CE-AI MANAGED BLOCK` from `.grok/rules/compound-engineering.md`.

### R4: Uninstallation & Cleanup
- WHEN `uninstall --harness grok` is run THEN `ce-ai` sidecars (`codegraph`, `engram`) MUST be unregistered from `[mcp_servers]`.
- WHEN `[mcp_servers]` becomes empty THEN `config.toml` MUST NOT be deleted to preserve user configuration and authentication credentials.
- WHEN `uninstall --harness grok` is run THEN `<harness_dir>/skills/` MUST be cleaned up without deleting user custom skills outside managed scope.

### R5: Skills Directory & Health Inspection
- WHEN `install` or `sync` runs for Grok harness THEN managed skills MUST be copied to `<harness_dir>/skills/<name>/SKILL.md`.
- WHEN `ce-ai doctor` or `ce-ai status` runs THEN Grok sidecars and skills status MUST be correctly reported.
