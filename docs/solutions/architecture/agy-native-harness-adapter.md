---
module: harness
tags: [agy, gemini, antigravity, harness, adapter, json, mcp, rules, google]
problem_type: architectural_refactor
---

# Solution: Google Antigravity (agy) Native Harness Adapter (Issue #179)

## Problem
Previously, `ce-ai` treated Google Antigravity (`agy`) as a generic JSON harness pointing to `~/.gemini/antigravity-cli/antigravity.json`. Official Google Antigravity documentation specifies `~/.gemini` as the root directory (overridden by `$ANTIGRAVITY_CONFIG_DIR` or `$GEMINI_HOME`), `~/.gemini/config/mcp_config.json` for global MCP configuration (`mcpServers` JSON object), `~/.gemini/config/skills/` for skills, and project rules under `.agents/rules/compound-engineering.md` and `GEMINI.md`. Furthermore, remote MCP servers in `mcp_config.json` MUST use `serverUrl` (and `headers`), NOT `url`.

## Solution Details
1. **Native Directory & Configuration**: Implemented `AgyAdapter` in `src/harness/agy.rs` targeting `~/.gemini/config/mcp_config.json` (or `$ANTIGRAVITY_CONFIG_DIR`/`$GEMINI_HOME`).
2. **Environment Override**: Supported `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` for harness directory resolution with thread-safe test environment locking via `HARNESS_ENV_LOCK`.
3. **Structured JSON Server Registration**:
   - `register_agy_mcp_server` updates `mcpServers.<name>` entries (`command`, `args`, `env`), preserving unmanaged user entries, remote `serverUrl` entries, and top-level JSON keys.
   - Resets `server_url` to `None` when registering a local stdio command server.
   - `unregister_agy_mcp_server` removes specified sidecar entries.
4. **Project Rule Adoption**: `init-prj` creates or updates `.agents/rules/compound-engineering.md` and project root `GEMINI.md` with `CE-AI MANAGED BLOCK`.
5. **Clean Uninstallation**: Unregisters `ce-ai` sidecars and removes `~/.gemini/config/skills/` and legacy `antigravity.json` while preserving user custom servers and custom skills.

## Verification
- Unit tests in `src/harness/agy.rs` verifying `mcpServers` JSON schema manipulation, `serverUrl` preservation, zero OpenCode key leaks, and thread safety under parallel execution.
- CLI integration tests in `tests/cli.rs` (`install_agy_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `init_prj_agy_writes_and_deinits_rules`, `uninstall_agy_harness_clean_install_lifecycle`, `uninstall_agy_harness_cleans_native_dir_artifacts_and_preserves_user_configs`).
- 100% green quality gates (137 unit tests, 73 integration tests).
