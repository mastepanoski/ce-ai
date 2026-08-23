# Implementation Plan: Grok Native Harness Adapter (Issue #176)

- **Goal**: Implement native xAI Grok Build CLI harness adapter in `ce-ai` targeting `~/.grok/config.toml` (TOML format), `~/.grok/skills/`, and `.grok/rules/compound-engineering.md`.

## Proposed Work Steps

### Step 1: Create `src/harness/grok.rs`
- Define `GrokAdapter` implementing `HarnessAdapter`.
- Implement `register_grok_mcp_server`, `unregister_grok_mcp_server`, `update_grok_rule_md`, and `strip_managed_block`.
- Add unit tests verifying `[mcp_servers]` TOML table manipulation, user config preservation, clean env map replacement, and managed comment block injection/stripping.

### Step 2: Register Module & Harness Mapping
- In `src/harness/mod.rs`:
  - Update `HarnessKind::Grok.harness_dir(home_dir)` to return `$GROK_HOME` if set, otherwise `home_dir.join(".grok")`.
  - Update `HarnessKind::Grok.config_path(base_dir)` to return `GrokAdapter.default_config_path(base_dir)`.
  - Update `is_installed_on_host` and `is_ce_installed` for `HarnessKind::Grok`.
- In `src/harness/generic_json.rs`:
  - Remove `HarnessKind::Grok` from `GenericJsonAdapter`.

### Step 3: Wire Subcommands
- `src/commands/install.rs`: Provision Grok `config.toml` with `codegraph` and `engram` in `[mcp_servers]`, copy managed skills into `<harness_dir>/skills/`.
- `src/commands/tools.rs`: Add Grok TOML server registration on `ce-ai tools install <tool>`.
- `src/commands/init_prj.rs` & `src/commands/deinit_prj.rs`: Add `.grok/rules/compound-engineering.md` adoption and de-adoption.
- `src/commands/sync.rs`: Reconcile Grok `[mcp_servers]` drift.
- `src/commands/uninstall.rs`: Unregister `ce-ai` sidecars from `[mcp_servers]` and clean skills directory, keeping user TOML settings intact.
- `src/commands/doctor.rs` & `src/commands/status.rs`: Update health check and status probing for Grok native sidecars.
- `src/state/backups.rs`: Tag Grok backups with `grok-` prefix.

### Step 4: CLI Integration Tests
- In `tests/cli.rs`:
  - `install_grok_harness_writes_to_native_dir_and_leaves_opencode_pristine`
  - `init_prj_grok_writes_and_deinits_rules_md`
  - `uninstall_grok_harness_cleans_native_dir_artifacts_and_preserves_user_configs`

### Step 5: Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### Step 6: Code Review & Knowledge Capture
- Run `ce-code-review` panel.
- Document solution in `docs/solutions/architecture/grok-native-harness-adapter.md`.
- Save Engram memory observation.

### Step 7: Release Shipping
- Bump version to `1.13.0` in `Cargo.toml` and `Formula/ce-ai.rb`.
- Update `CHANGELOG.md`.
- Commit, push, open PR, wait for 100% green CI matrix, merge, tag `v1.13.0`, release, close Issue #176.
