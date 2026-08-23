# Specification: GitHub Copilot Native Harness Adapter

## Acceptance Criteria

### R1: Harness Home & Config Path Resolution
- WHEN `harness_dir(HarnessKind::Copilot)` is called THEN it MUST return `$COPILOT_CONFIG_DIR` if set, otherwise `$HOME/.copilot`.
- WHEN `default_config_path` is called THEN it MUST return `$COPILOT_CONFIG_DIR/mcp-config.json` if set, otherwise `$HOME/.copilot/mcp-config.json`.
- WHEN `is_installed_on_host` is called for `HarnessKind::Copilot` THEN it MUST check if `$COPILOT_CONFIG_DIR` exists or if `$HOME/.copilot` exists.
- WHEN `is_ce_installed` is called for `HarnessKind::Copilot` THEN it MUST check if `mcp-config.json` contains `mcpServers` sidecars or if `skills/` exists under the harness directory.

### R2: Native JSON MCP Server Configuration
- WHEN `ce-ai install --harness copilot` or `ce-ai tools install <tool>` runs THEN sidecar MCP servers MUST be registered under `mcpServers` in JSON format (`command`, `args`, `env`).
- WHEN writing `~/.copilot/mcp-config.json` THEN `ce-ai` MUST NOT write OpenCode keys (`plugin`, `skills.paths`).
- WHEN updating `~/.copilot/mcp-config.json` THEN `ce-ai` MUST preserve all unmanaged top-level JSON keys and user `mcpServers` entries.
- WHEN backups are created for Copilot THEN `backups.rs` MUST format snapshot file names with the `copilot-` prefix.

### R3: Project Rule Adoption
- WHEN `ce-ai init-prj` runs in a Copilot-enabled project THEN project rules MUST be updated in `.github/copilot-instructions.md`.
- WHEN `ce-ai init-prj` runs THEN it MUST inject or update the demarcated `CE-AI MANAGED BLOCK`.
- WHEN `ce-ai deinit-prj` is run THEN it MUST strip the demarcated `CE-AI MANAGED BLOCK` from `.github/copilot-instructions.md`.

### R4: Uninstallation & Cleanup
- WHEN `uninstall --harness copilot` is run THEN `ce-ai` sidecars (`codegraph`, `engram`) MUST be unregistered from `mcpServers`.
- WHEN `mcpServers` becomes empty THEN `mcp-config.json` MUST NOT be deleted to preserve user configuration and authentication credentials.
- WHEN `uninstall --harness copilot` is run THEN `<harness_dir>/skills/` MUST be cleaned up.

### R5: Skills Directory & Health Inspection
- WHEN `install` or `sync` runs for Copilot harness THEN managed skills MUST be copied to `<harness_dir>/skills/<name>/SKILL.md`.
- WHEN `ce-ai doctor` or `ce-ai status` runs THEN Copilot sidecars and skills status MUST be correctly reported.
