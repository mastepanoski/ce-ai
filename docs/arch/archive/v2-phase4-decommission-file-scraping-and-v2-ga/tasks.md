# Tasks: Phase 4 — Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0 GA)

## Work Units & Changed-Line Estimates

- [x] **Unit 1: Decommission File Scraping & Watch Loop in Command Surfaces** (~180 LOC)
  - [x] Add deprecation warning to `src/commands/install.rs` directing users to `init-prj` and `fleet pin / sync`.
  - [x] Add deprecation warning to `src/commands/upgrade.rs` directing users to `fleet pin / sync`.
  - [x] Add deprecation warning to `src/commands/sync.rs` and update `--watch` handler to warn about dual sources of truth.
  - [x] Mark `extract_to_source` and `find_source_root` in `src/source/archive.rs` as deprecated/decommissioned while preserving binary self-update security primitives.

- [x] **Unit 2: CLI & Integration Test Updates** (~150 LOC)
  - [x] Update unit and integration tests in `src/commands/tests/install.rs` and `src/commands/tests/sync.rs` to verify deprecation advisories.
  - [x] Ensure `tests/cli.rs` passes with all deprecation notices emitted cleanly.

- [x] **Unit 3: Documentation & Identity Overhaul (v2.0 GA)** (~120 LOC)
  - [x] Update `README.md` to define CE-AI as the *Cross-Host Operational Companion for Compound Engineering* (strictly ≤ 100 lines).
  - [x] Update Quick Start in `README.md` to showcase `init-prj`, `fleet pin`, `fleet sync`, and `workflow status`.
  - [x] Update description in `Cargo.toml`.

- [x] **Unit 4: Monotonic Concept Accretion & Solution Capture** (~140 LOC)
  - [x] Pre-read `CONCEPTS.md` and append 3 new concepts monotonically:
    - [x] `Cross-Host Operational Companion (v2 GA)`
    - [x] `Decommissioned File Scraping`
    - [x] `Native Host Packaging Sovereignty`
  - [x] Verify concept integrity with `python3 scripts/validate-concepts.py` and `cargo run -- doc lint --strict`.
  - [x] Author solution document in `docs/solutions/architecture/ce-ai-v2-ga-operational-companion-pivot.md`.
  - [x] Validate solution frontmatter with `cargo run -- doc lint --strict`.

- [x] **Unit 5: SemVer Bump & Changelog Update** (~80 LOC)
  - [x] Bump `version = "2.0.0"` in `Cargo.toml`.
  - [x] Update `CHANGELOG.md` with complete v2.0.0 release notes covering all 4 phases.

- [x] **Unit 6: Empirical Verification & Quality Gates** (0 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e`.
