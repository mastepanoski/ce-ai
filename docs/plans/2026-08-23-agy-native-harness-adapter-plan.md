# Implementation Plan: Google Antigravity (agy) Native Harness Adapter (Issue #179)

Convert Google Antigravity (`agy`) from generic JSON harness into a first-class native harness adapter targeting `~/.gemini/config/mcp_config.json` (`mcpServers` JSON object), `~/.gemini/config/skills/`, and project rule adoption under `.agents/rules/compound-engineering.md` and `GEMINI.md`.

## User Review Required

> [!NOTE]
> Google Antigravity uses `~/.gemini/config/mcp_config.json` for global MCP configuration, with `serverUrl` for remote SSE/HTTP servers. It supports `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` environment overrides.

## Proposed Changes

### 1. New Native Module `src/harness/agy.rs`
- Define `AgyMcpConfig` and `AgyMcpServer` structs with `serverUrl` support.
- Implement `register_agy_mcp_server` and `unregister_agy_mcp_server` using `write_atomic`.
- Add unit tests for serialization, remote `serverUrl` preservation, zero OpenCode leaks, and `$ANTIGRAVITY_CONFIG_DIR`/`$GEMINI_HOME` thread locks via `HARNESS_ENV_LOCK`.

### 2. Wire `HarnessKind::Agy` in Core Harness Dispatch (`src/harness/mod.rs`)
- Update `harness_dir` to check `$ANTIGRAVITY_CONFIG_DIR` -> `$GEMINI_HOME` -> `<home_dir>/.gemini`.
- Update `config_path` to `harness_dir.join("config").join("mcp_config.json")`.
- Implement `AgyAdapter` struct implementing `HarnessAdapter`.

### 3. Subcommand Wiring
- `src/commands/install.rs`: Native MCP registration in `mcp_config.json` and skills in `~/.gemini/config/skills/`.
- `src/commands/tools.rs`: Tool registration in `mcp_config.json`.
- `src/commands/init_prj.rs` & `deinit_prj.rs`: Project rule adoption in `.agents/rules/compound-engineering.md` and `GEMINI.md`.
- `src/commands/sync.rs` & `backups.rs`: Native sync drift checking and backup filtering for `agy`.
- `src/commands/uninstall.rs`: Sidecar unregistration, skills cleanup, and legacy `antigravity.json` cleanup with warning emission.
- `src/harness/generic_json.rs`: Remove legacy `HarnessKind::Agy` generic JSON mapping.

### 4. Integration Tests (`tests/cli.rs`)
- `install_agy_harness_writes_to_native_dir_and_leaves_opencode_pristine`
- `init_prj_agy_writes_and_deinits_rules`
- `uninstall_agy_harness_clean_install_lifecycle`
- `uninstall_agy_harness_cleans_native_dir_artifacts_and_preserves_user_configs`

## Verification Plan

### Automated Tests
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
