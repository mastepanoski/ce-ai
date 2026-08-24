---
module: harness
tags: [kimi, harness, adapter, json, mcp, rules, moonshot]
problem_type: architectural_refactor
---

# Solution: Kimi Code CLI Native Harness Adapter (Issue #178)

## Problem
Previously, `ce-ai` treated Kimi Code CLI (`kimi`) as a generic JSON harness at `~/.kimi/config.json`. Official Moonshot AI documentation specifies `~/.kimi-code/` as the native configuration root (overridden by `$KIMI_CODE_HOME`), `~/.kimi-code/mcp.json` for MCP server registration (`mcpServers` JSON object), `~/.kimi-code/skills/` for agent skills, and project rules under `AGENTS.md` or `.kimi-code/rules/compound-engineering.md`.

## Solution Details
1. **Native Directory & Configuration**: Implemented `KimiAdapter` in `src/harness/kimi.rs` targeting `~/.kimi-code/mcp.json` (or `$KIMI_CODE_HOME/mcp.json`).
2. **Environment Override**: Supported `$KIMI_CODE_HOME` for harness directory resolution with thread-safe test environment locking via `HARNESS_ENV_LOCK`.
3. **Structured JSON Server Registration**:
   - `register_kimi_mcp_server` updates `mcpServers.<name>` entries (`command`, `args`, `env`), preserving unmanaged user entries and top-level JSON keys.
   - `unregister_kimi_mcp_server` removes specified sidecar entries.
4. **Project Rule Adoption**: `init-prj` creates or updates `.kimi-code/rules/compound-engineering.md` and project `AGENTS.md` with `CE-AI MANAGED BLOCK`.
5. **Clean Uninstallation**: Unregisters `ce-ai` sidecars and removes `~/.kimi-code/skills/` while preserving user custom servers and custom skills.

## Verification
- Unit tests in `src/harness/kimi.rs` verifying `mcpServers` JSON schema manipulation, zero OpenCode key leaks, thread safety under parallel execution, and clean env map replacement.
- CLI integration tests in `tests/cli.rs` (`install_kimi_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `init_prj_kimi_writes_and_deinits_agents_md`, `uninstall_kimi_harness_clean_install_lifecycle`, `uninstall_kimi_harness_cleans_native_dir_artifacts_and_preserves_user_configs`).
- 100% green quality gates (133 unit tests, 69 integration tests).
