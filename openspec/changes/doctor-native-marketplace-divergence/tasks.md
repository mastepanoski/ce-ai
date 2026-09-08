# Tasks: Detect Claude Code Native Marketplace Plugin Divergence

Work-unit changed-line estimates total: ~210 LOC target (~200 LOC policy).

- [x] **Task 1: Core divergence models & pure functions in `src/harness/claude.rs`** (~70 LOC)
  - [x] 1.1 Define `ClaudeMarketplaceDivergence` struct and internal deserialization types (`NativeInstalledPlugins`, `NativePluginEntry`).
  - [x] 1.2 Implement `normalize_plugin_version(raw: &str) -> &str`.
  - [x] 1.3 Implement `check_claude_marketplace_divergence(state: &State, cwd: &Path, claude_dir: &Path) -> Vec<ClaudeMarketplaceDivergence>`.

- [x] **Task 2: Unit testing matrix in `src/harness/tests/claude.rs`** (~70 LOC)
  - [x] 2.1 Test missing file, empty file, and malformed JSON resilience.
  - [x] 2.2 Test harness absence guard (no `claude` harness in `state.installed_harnesses`).
  - [x] 2.3 Test version matching: verify no divergence when normalized versions match (`compound-engineering-v3.24.0` vs `3.24.0`).
  - [x] 2.4 Test user scope divergence: verify detection and proper field mapping.
  - [x] 2.5 Test project / local scope applicability: verify entry included when `cwd` matches `projectPath`, and excluded when `cwd` does not match.

- [x] **Task 3: Doctor CLI integration & advisory output** (~30 LOC)
  - [x] 3.1 Call `check_claude_marketplace_divergence` from `doctor::run`.
  - [x] 3.2 Format and print advisory `doctor-info:` notice with exact update command.
  - [x] 3.3 Ensure zero mutation of disk files and verify exit code remains 0.

- [x] **Task 4: CLI integration test & verification quality gates** (~40 LOC)
  - [x] 4.1 Add hermetic test in `tests/cli.rs` verifying `ce-ai doctor` CLI output contains advisory `doctor-info:` and exits with code 0.
  - [x] 4.2 Run `cargo fmt --check`.
  - [x] 4.3 Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] 4.4 Run `cargo test`.
  - [x] 4.5 Run `make e2e`.
