# Implementation Plan: Codex Native Harness Adapter (Issue #175)

- **Goal**: Implement native OpenAI Codex CLI / harness adapter in `ce-ai` targeting `~/.codex/config.toml` (TOML format), `~/.codex/skills/`, and `AGENTS.md`.

## Proposed Work Steps

### Step 1: Create `src/harness/codex.rs`
- Define `CodexAdapter` implementing `HarnessAdapter`.
- Implement `register_codex_mcp_server`, `unregister_codex_mcp_server`, `update_codex_agents_md`, and `strip_managed_block`.
- Add unit tests verifying TOML table creation, user config preservation, and managed comment block injection/stripping.

### Step 2: Register Module & Harness Mapping
- In `src/harness/mod.rs`:
  - Declare `pub mod codex;`
  - Update `HarnessKind::Codex.harness_dir(home_dir)` to return `$CODEX_CONFIG_DIR` if set, otherwise `home_dir.join(".codex")`.
  - Update `HarnessKind::Codex.config_path(base_dir)` to return `CodexAdapter.default_config_path(base_dir)`.
  - Update `is_installed_on_host` and `is_ce_installed` for `HarnessKind::Codex`.

### Step 3: Wire Subcommands
- `src/commands/install.rs`: Provision Codex `config.toml` with `codegraph` and `engram` in `[mcp_servers]`, copy managed skills into `~/.codex/skills/`.
- `src/commands/tools.rs`: Add Codex TOML server registration on `ce-ai tools install <tool>`.
- `src/commands/init_prj.rs` & `src/commands/deinit_prj.rs`: Add `AGENTS.md` / `.codex/AGENTS.md` adoption and de-adoption.
- `src/commands/sync.rs`: Reconcile Codex TOML `[mcp_servers]` drift.
- `src/commands/uninstall.rs`: Unregister `ce-ai` sidecars from `[mcp_servers]`, keeping user TOML settings intact.
- `src/state/backups.rs`: Tag Codex backups with `codex-` prefix.

### Step 4: CLI Integration Tests
- In `tests/cli.rs`:
  - `install_codex_harness_writes_to_native_dir_and_leaves_opencode_pristine`
  - `init_prj_codex_writes_agents_md`
  - `uninstall_codex_harness_cleans_native_dir_artifacts`

### Step 5: Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### Step 6: Code Review & Knowledge Capture
- Run `ce-code-review` panel.
- Document solution in `docs/solutions/architecture/codex-native-harness-adapter.md`.
- Save Engram memory observation.

### Step 7: Release Shipping
- Bump version to `1.11.0` in `Cargo.toml` and `Formula/ce-ai.rb`.
- Update `CHANGELOG.md`.
- Commit, push, open PR, wait for 100% green CI matrix, merge, tag `v1.11.0`, release, close Issue #175.
