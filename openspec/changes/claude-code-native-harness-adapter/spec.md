# Specification: Claude Code Native Harness Adapter (Issue #174)

## Requirements

### R1: Native Harness Directory & Default Config Path
- WHEN `HarnessKind::Claude.harness_dir(&home)` is called THEN it MUST return `CLAUDE_CONFIG_DIR` if set, else `home.join(".claude")`.
- WHEN `ClaudeAdapter::default_config_path(&home)` is called THEN:
  - If `~/.claude/settings.json` exists and contains `"mcpServers"`, it MUST return `~/.claude/settings.json`.
  - Otherwise, it MUST return `~/.claude.json`.

### R2: Native `mcpServers` Stdio Schema & User Data Preservation
- WHEN `register_claude_mcp_server` is invoked THEN it MUST write entries under `mcpServers.<name>` using `type: "stdio"`, `command`, `args`, and `env`.
- WHEN `ce-ai tools install <tool>` is executed for Claude harness THEN it MUST register the companion tool under `mcpServers` in the active Claude config path.
- WHEN pre-existing user settings or non-CE MCP servers exist THEN they MUST be preserved intact via `#[serde(flatten)] pub extra`.
- WHEN `install` or `sync` targets Claude harness THEN OpenCode keys (`plugin`, `skills.paths`) MUST NOT be written to Claude config files.
- WHEN atomic write operations encounter serialization or IO failures THEN the target file MUST remain unmutated and a `CeError::Runtime` / `CeError::Io` error MUST be returned.

### R3: Project Directive Adoption & De-adoption (`init-prj` / `deinit-prj`)
- WHEN `ce-ai init-prj` is run THEN it MUST resolve project rule file path by priority:
  1. `./CLAUDE.md` if present
  2. `.claude/CLAUDE.md` if `.claude/` exists
  3. `./CLAUDE.md` default
- WHEN `ce-ai init-prj` runs THEN it MUST inject or update the demarcated `CE-AI MANAGED BLOCK`.
- WHEN `ce-ai deinit-prj` is run THEN it MUST strip the demarcated `CE-AI MANAGED BLOCK` from the project rule file.

### R4: Uninstallation & Cleanup
- WHEN `uninstall --harness claude` is run THEN `ce-ai` sidecars (`codegraph`, `engram`) MUST be unregistered from `mcpServers`.
- WHEN `mcpServers` becomes empty THEN the `mcpServers` block MAY be cleared, but the configuration file (`~/.claude.json` / `settings.json`) MUST NOT be deleted even when empty to preserve user application state (OAuth sessions, project trust, and preferences).

### R5: Skills Directory & Asset Placement
- WHEN `install` or `sync` runs for Claude harness THEN managed skills MUST be copied to `~/.claude/skills/<name>/SKILL.md`.
