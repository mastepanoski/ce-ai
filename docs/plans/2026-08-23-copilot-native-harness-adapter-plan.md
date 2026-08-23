# Implementation Plan: GitHub Copilot Native Harness Adapter (Issue #177)

- **Goal**: Implement native GitHub Copilot CLI / extension harness adapter in `ce-ai` targeting `~/.copilot/mcp-config.json` (JSON format), `~/.copilot/skills/`, and `.github/copilot-instructions.md`.

## Proposed Work Steps

### Step 1: Create `src/harness/copilot.rs`
- Define `CopilotAdapter` implementing `HarnessAdapter`.
- Implement `register_copilot_mcp_server`, `unregister_copilot_mcp_server`, `update_copilot_instructions_md`, and `strip_managed_block`.
- Add unit tests verifying `mcpServers` object creation, user config preservation, and managed comment block injection/stripping.

### Step 2: Register Module & Harness Mapping
- In `src/harness/mod.rs`:
  - Update `HarnessKind::Copilot.harness_dir(home_dir)` to return `$COPILOT_CONFIG_DIR` if set, otherwise `home_dir.join(".copilot")`.
  - Update `HarnessKind::Copilot.config_path(base_dir)` to return `CopilotAdapter.default_config_path(base_dir)`.
  - Update `is_installed_on_host` and `is_ce_installed` for `HarnessKind::Copilot`.

### Step 3: Wire Subcommands
- `src/commands/install.rs`: Provision Copilot `mcp-config.json` with `codegraph` and `engram` in `mcpServers`, copy managed skills into `~/.copilot/skills/`.
- `src/commands/tools.rs`: Add Copilot JSON server registration on `ce-ai tools install <tool>`.
- `src/commands/init_prj.rs` & `src/commands/deinit_prj.rs`: Add `.github/copilot-instructions.md` adoption and de-adoption.
- `src/commands/sync.rs`: Reconcile Copilot `mcpServers` drift.
- `src/commands/uninstall.rs`: Unregister `ce-ai` sidecars from `mcpServers` and clean skills directory, keeping user JSON settings intact.
- `src/state/backups.rs`: Tag Copilot backups with `copilot-` prefix.

### Step 4: CLI Integration Tests
- In `tests/cli.rs`:
  - `install_copilot_harness_writes_to_native_dir_and_leaves_opencode_pristine`
  - `init_prj_copilot_writes_and_deinits_copilot_instructions`
  - `uninstall_copilot_harness_cleans_native_dir_artifacts_and_preserves_user_configs`

### Step 5: Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### Step 6: Code Review & Knowledge Capture
- Run `ce-code-review` panel.
- Document solution in `docs/solutions/architecture/copilot-native-harness-adapter.md`.
- Save Engram memory observation.

### Step 7: Release Shipping
- Bump version to `1.12.0` in `Cargo.toml` and `Formula/ce-ai.rb`.
- Update `CHANGELOG.md`.
- Commit, push, open PR, wait for 100% green CI matrix, merge, tag `v1.12.0`, release, close Issue #177.
