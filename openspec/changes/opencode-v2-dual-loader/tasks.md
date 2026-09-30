# Tasks: OpenCode V1/V2 Dual Plugin Loader, Version-Aware Detection & Native Commands

- [ ] **Unit 1 (~180 LOC):** Dual plugin loader in `.opencode/plugins/compound-engineering.js`
  - Implement dual export: V1 `CompoundEngineeringPlugin` (named and `default.server`) and V2 `default.setup(ctx)` with `id: "compound-engineering"`.
  - Add dynamic `skillsDir` resolution checking candidate paths (`../../skills`, `../compound-engineering/skills`, `../skills`).
  - Preserve `spawnSync` + `getRepoState()` workflow resume injection across `session.created`, `session.idle`, `context`, and `compaction`.
  - *Verification*: Node/Bun ESM validation verifying `default.id === "compound-engineering"`, `typeof default.setup === "function"`, and `typeof default.server === "function"`.

- [ ] **Unit 2 (~190 LOC):** Version-aware detection and config key agnosticism in `src/opencode/plugins.rs` and `config.rs`
  - Add `OpenCodeVersion` enum (`V1`, `V2`, `Unknown`) and runtime version probe via `opencode --version`.
  - Update `is_valid_loader_content` to validate V1, V2, and dual loader shapes against target OpenCode version.
  - Update `has_session_start_plugin` to accept both `"plugins"` and `"plugin"` in `opencode.json`.
  - Update `ensure_session_start_plugin` to check `is_valid_loader_content` before skipping write, self-repairing stale loaders.
  - Update `merge_plugin` in `src/opencode/config.rs` and `remove_session_start_plugin` to handle both `"plugins"` and `"plugin"`.
  - *Verification*: `cargo check` and `cargo test --lib` for plugin and config modules.

- [ ] **Unit 3 (~180 LOC):** Native command materialization & OpenCode V2 plugin placement
  - Implement `ensure_managed_commands` and `remove_managed_commands` under `<config_dir>/commands/` for user-invocable skills with `<!-- ce-ai:managed-command -->` guards.
  - Add plugin auto-discovery placement (copy/symlink to `<config_dir>/plugins/compound-engineering.js`) when V2 is active.
  - Integrate command management into `ce-ai install`, `ce-ai sync`, and `ce-ai uninstall`.
  - *Verification*: `cargo test` verifying command file generation, idempotence on second sync, and clean removal on uninstall.

- [ ] **Unit 4 (~170 LOC):** Test coverage, SemVer bump, and live verification
  - Add tests in `src/opencode/tests/plugins.rs` for V1 key, V2 key, version detection, dual loader shape, and command lifecycle.
  - Bump SemVer to `1.74.0` in `Cargo.toml` and document in `CHANGELOG.md`.
  - Verify live with `opencode api get /api/plugin`, `opencode api get /api/command`, and `cargo run -- doctor`.
  - Run full suite: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`, and `cargo run -- doc lint --strict`.
