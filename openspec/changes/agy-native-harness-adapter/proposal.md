# Proposal: Google Antigravity (agy) Native Harness Adapter (Issue #179)

## Problem Statement
Previously, `ce-ai` treated Google Antigravity (`agy`) as a generic JSON harness pointing to `~/.gemini/antigravity-cli/antigravity.json`. However, official Google Antigravity documentation specifies:
- Global MCP configuration lives at `~/.gemini/config/mcp_config.json` (under top-level key `mcpServers`).
- Stdio MCP servers use `command`, `args`, `env`; remote HTTP/SSE servers MUST use `serverUrl` (and optional `headers`). Legacy `url`/`httpUrl` keys are rejected by Antigravity.
- Global skills live at `~/.gemini/config/skills/` (and `~/.gemini/antigravity-cli/skills/`).
- Project rules live at `.agents/rules/` (e.g. `.agents/rules/compound-engineering.md`) or global `~/.gemini/GEMINI.md`.
- Environment variable overrides `$ANTIGRAVITY_CONFIG_DIR` or `$GEMINI_HOME` allow relocating `~/.gemini`.

## In-Scope
1. Native `AgyAdapter` in `src/harness/agy.rs` implementing `HarnessAdapter`.
2. Supporting `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` environment variable overrides for native directory resolution.
3. Native MCP server registration in `~/.gemini/config/mcp_config.json` under `mcpServers` JSON object format, using `serverUrl` for remote servers if present.
4. Skills installation into `~/.gemini/config/skills/` (and `~/.gemini/antigravity-cli/skills/`).
5. Project rule adoption in `.agents/rules/compound-engineering.md` and `GEMINI.md`.
6. Clean uninstallation of `ce-ai` sidecars and skills while preserving user custom servers and custom skills.
7. Zero OpenCode key leaks (`plugin`, `skills.paths`).

## Out-of-Scope
- Modifying Antigravity CLI binary (`agy`) or Antigravity IDE settings.json.

## Risks & Mitigations
- **Risk**: Test race conditions when setting `$ANTIGRAVITY_CONFIG_DIR` or `$GEMINI_HOME` during parallel `cargo test` runs.
- **Mitigation**: Use process-wide `crate::harness::tests::HARNESS_ENV_LOCK` mutex in all tests mutating environment variables.

## Success Criteria
1. `install --harness agy` creates valid `~/.gemini/config/mcp_config.json` with `codegraph` and `engram` under `mcpServers` and populates `~/.gemini/config/skills/`.
2. `uninstall --harness agy` unregisters `ce-ai` sidecars and cleans `skills/` without touching user custom servers.
3. Zero OpenCode key leaks (`plugin`, `skills.paths`).
4. 100% green CI matrix across Linux, macOS, and Windows.
