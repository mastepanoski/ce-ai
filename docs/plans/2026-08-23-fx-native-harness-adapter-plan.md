# Implementation Plan: fx Native Harness Adapter

Implement native `FxAdapter` targeting Vercel Labs' `fx` coding agent (`~/.fx/`, `$FX_HOME`, `mcp` root key in `~/.fx/mcp.json` with array-form commands, and skills under `~/.fx/skills/`).

## User Review Required
> [!IMPORTANT]
> `fx` MCP configuration uses root key `mcp` (`{"mcp": {"<name>": {"type": "local", "command": ["<cmd>", "<args>..."], "environment": {}}}}`), NOT `mcpServers`. Native harness directory is `~/.fx/` (overridden by `$FX_HOME`).

## Proposed Changes

### 1. Harness Adapter (`src/harness/fx.rs` & `src/harness/mod.rs`)
- Create `src/harness/fx.rs` implementing `FxAdapter`:
  - `FxMcpConfig` serde struct (`mcp: BTreeMap<String, FxMcpServer>`, `extra`).
  - `FxMcpServer` serde struct (`r#type: Option<String>`, `command: Vec<String>`, `environment: BTreeMap<String, String>`, `extra`).
  - `register_fx_mcp_server(config_path, name, command, args, env)` helper.
  - `unregister_fx_mcp_server(config_path, name)` helper.
  - `harness_dir(home_dir)` -> check `$FX_HOME` if set and non-empty, else `home_dir.join(".fx")`.
  - `default_config_path(home_dir)` -> `harness_dir(home_dir).join("mcp.json")`.
  - `canonical_instruction_file()` -> `AGENTS.md`.
  - `derived_stub_files()` -> `vec![.fx/AGENTS.md]`.
- Wire `HarnessKind::Fx` across `src/harness/mod.rs`, `src/commands/install.rs`, `src/commands/tools.rs`, `src/commands/uninstall.rs`, `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, and remove generic JSON mapping from `src/harness/generic_json.rs`.

### 2. Verification Plan
- Unit tests in `src/harness/fx.rs`.
- Integration tests in `tests/cli.rs`: `install_fx_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `uninstall_fx_harness_cleans_native_dir_artifacts_and_preserves_user_configs`.
- Quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`).
