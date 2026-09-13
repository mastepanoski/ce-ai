# Tasks: Detect Kimi Code Native Plugin Manager Divergence

Work-unit changed-line estimates total: ~310 LOC (~200 LOC/work-unit policy applied to the code-bearing units; test/docs units are additive verification).

- [x] **Task 1: Core divergence models & pure functions in `src/harness/kimi.rs`** (~90 LOC)
  - [x] 1.1 Define `KimiMarketplaceDivergence`, `KimiOrphanManagedTree`, and internal deserialization types (`NativeInstalledJson`, `NativeInstalledPlugin`).
  - [x] 1.2 Implement `check_kimi_marketplace_divergence(state, cwd, kimi_dir) -> Vec<KimiMarketplaceDivergence>` with harness guard, `enabled` filter, native version resolution (`package.json` → `plugin.json`), and normalized comparison reusing `claude::normalize_plugin_version`.
  - [x] 1.3 Implement `check_kimi_orphan_managed_tree(kimi_dir) -> Option<KimiOrphanManagedTree>` with `install-manifest.json` marker and `config.toml` `extra_skill_dirs` reference check.

- [x] **Task 2: Unit testing matrix in `src/harness/tests/kimi.rs`** (~130 LOC)
  - [x] 2.1 Missing/empty/malformed `installed.json` resilience → empty vector (TDD: red first).
  - [x] 2.2 Harness absence guard (state has opencode but not kimi) → empty vector.
  - [x] 2.3 Enabled native plugin with divergent version (fixture: `installed.json` + `root/package.json`) → exactly one divergence with correct field mapping.
  - [x] 2.4 Disabled native plugin and version-equal native plugin → no divergence; version resolution falls back to `plugin.json`.
  - [x] 2.5 Orphan tree matrix: managed tree + unreferenced/empty/missing `config.toml` → `Some`; referenced `extra_skill_dirs` → `None`; no managed tree → `None`.
  - [x] 2.6 All fixtures use `tempfile::TempDir`; never read from or write to the real `~/.kimi-code`.

- [x] **Task 3: Doctor CLI integration (non-blocking)** (~50 LOC)
  - [x] 3.1 Call both kimi probes from `doctor::run` after the Claude marketplace probe, using `HarnessKind::Kimi.harness_dir(&home_dir)`.
  - [x] 3.2 Emit `doctor-info:` divergence lines and `doctor-warn:` orphan line per the design output contract.
  - [x] 3.3 Verify zero disk mutation and unchanged exit behavior.

- [x] **Task 4: Doctor integration test in `src/commands/tests/doctor.rs`** (~40 LOC)
  - [x] 4.1 Hermetic temp-home test: divergent native kimi plugin + orphan managed tree → `run(&ctx, &Args::default())` still returns `Ok(())`.

- [x] **Task 5: Docs & verification quality gates** (~40 LOC)
  - [x] 5.1 Update `CHANGELOG.md` (Unreleased, Keep a Changelog).
  - [x] 5.2 Run `cargo fmt --check`.
  - [x] 5.3 Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] 5.4 Run `cargo test`.
