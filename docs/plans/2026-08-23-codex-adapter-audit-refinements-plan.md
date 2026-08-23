# Implementation Plan: Codex Adapter Audit Refinements

- **Goal**: Resolve audit findings for Issue #175 (Codex Native Adapter) by adopting `CODEX_HOME` env var, removing dead generic JSON mapping, cleanly replacing env maps during TOML server registration, and amending spec contracts.

## Proposed Work Steps

### Step 1: Replace `CODEX_CONFIG_DIR` with `CODEX_HOME`
- Update `harness_dir` in `src/harness/mod.rs`.
- Update `default_config_path` in `src/harness/codex.rs`.

### Step 2: Update TOML Env Registration Logic
- In `src/harness/codex.rs`, update `register_codex_mcp_server` to cleanly overwrite or remove `env` with the new environment map.

### Step 3: Remove Dead Generic JSON Code
- In `src/harness/generic_json.rs`, remove `HarnessKind::Codex` mapping and unit test assertion.

### Step 4: Amend OpenSpec Contracts & Docs
- Update `openspec/changes/codex-native-harness-adapter/spec.md` (R1 and R3) and `openspec/changes/codex-adapter-audit-refinements/spec.md` (R1 and R2) to document `CODEX_HOME` and `.codex/AGENTS.md` adoption contract.

### Step 5: Verification & Quality Gates
- Run unit and CLI tests.
- Run `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`.

### Step 6: Code Review & Shipping
- Run `ce-code-review` panel.
- Document in `docs/solutions/architecture/codex-native-harness-adapter.md`.
- Bump version to `1.12.1`, ship PR, wait for green CI, merge, tag `v1.12.1`.
