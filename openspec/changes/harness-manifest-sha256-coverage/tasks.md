# Tasks: SHA256 Manifest Coverage for Non-OpenCode Harnesses

Work-unit changed-line estimates total: ~420 LOC (~200 LOC/work-unit policy applied to code-bearing units; tests/docs are additive verification).

- [x] **Task 1: Harvest helper in `src/opencode/manifest.rs`** (~40 LOC)
  - [ ] 1.1 (TDD) Add failing unit tests in `src/opencode/tests/manifest.rs`: harvest returns sorted relative-path SHA256 entries; excludes `install-manifest.json`; empty/missing dir → empty vec.
  - [ ] 1.2 Implement `InstallManifest::harvest(managed_dir)` via recursive walk + `crate::state::diff::sha256_hex`.

- [x] **Task 2: Sync root-cause fix in `src/commands/sync.rs`** (~50 LOC)
  - [ ] 2.1 (TDD) Add regression test in `src/commands/tests/sync.rs`: `sync_with` with a `kimi` state entry and a pre-existing empty manifest under the temp-home kimi dir rewrites it with real harvested hashes (and preserves `installed_at`).
  - [ ] 2.2 Replace `files: vec![]` with `InstallManifest::harvest(&managed_dir)`; preserve prior `installed_at`/`config_mutations`; replace `let _ =` with `?`.

- [x] **Task 3: Multi-harness drift detection** (~120 LOC)
  - [ ] 3.1 `src/commands/workflow.rs`: `probe_manifest_drift_count` sums OpenCode + claude + kimi manifest drift (harness dirs resolved via `home_dir_from_ctx` + `HarnessKind::harness_dir`).
  - [ ] 3.2 `src/commands/doctor.rs`: after the OpenCode diff block, diff claude/kimi manifests when present and push `diff: <harness> <kind> <path>` findings.
  - [ ] 3.3 `src/commands/status.rs`: iterate opencode + claude + kimi manifests; keep `drift: none` / `drift: unknown (no install manifest)` contracts byte-identical; print `drift: <harness>: <kind> <path>` for non-OpenCode drift.

- [x] **Task 4: Drift detection tests** (~130 LOC)
  - [ ] 4.1 Doctor: tampered file under a temp-home claude managed tree (manifest with real hash) → finding `diff: claude modified <path>` and non-zero exit.
  - [ ] 4.2 Doctor: absent claude manifest → no claude finding (graceful degradation).
  - [ ] 4.3 Workflow: `probe_manifest_drift_count` counts tampered claude managed files via temp-home ctx.
  - [ ] 4.4 Harvest/symlink-safety and determinism covered in Task 1 tests.

- [x] **Task 5: Docs & verification quality gates** (~80 LOC)
  - [ ] 5.1 Update `CHANGELOG.md` (Unreleased, Keep a Changelog).
  - [ ] 5.2 Run `cargo fmt --check`.
  - [ ] 5.3 Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [ ] 5.4 Run `cargo test`.
  - [ ] 5.5 Audit for stale comments referencing empty manifests (`grep -rn "files: vec!\[\]" src/`).
