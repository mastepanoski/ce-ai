# Implementation Plan: Copilot Adapter Audit Refinements

- **Goal**: Refine native GitHub Copilot CLI adapter implementation based on audit feedback.

## Proposed Work Steps

### Step 1: OpenSpec Contract & Doc Review
- Author OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- Run `ce-doc-review` panel and incorporate feedback.

### Step 2: Implementation & Tests
- `src/harness/copilot.rs`: Update `register_copilot_mcp_server` to cleanly overwrite `server_entry.env = env.clone()`.
- `src/commands/uninstall.rs`: Replace `let _ = std::fs::remove_dir_all(&skills_dir);` with warning logging on failure.
- `openspec/changes/copilot-native-harness-adapter/design.md` & `spec.md`: Document `COPILOT_CONFIG_DIR`.
- `src/harness/copilot.rs`: Add unit test `replaces_env_map_cleanly_on_re_registration`.
- `src/commands/uninstall.rs`: Verify skills directory removal warning logging behavior.

### Step 3: Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`

### Step 4: Code Review & Shipping
- Run `ce-code-review` panel.
- Document solution in `docs/solutions/architecture/copilot-native-harness-adapter.md`.
- Save Engram observation.
- Bump SemVer to `v1.13.1` in `Cargo.toml` and `Formula/ce-ai.rb`, update `CHANGELOG.md`.
- Commit, push, open PR, wait for green CI matrix, merge, tag `v1.13.1`, release.
