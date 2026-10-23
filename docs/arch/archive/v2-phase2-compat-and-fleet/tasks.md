# Tasks: Phase 2 — CE Compatibility Layer & Fleet Subsystem

- [x] **Unit 1 (~180 LOC):** Implement Typed CE Compatibility Layer (`src/compat/`)
  - Create `src/compat/mod.rs`, `src/compat/docs.rs`, `src/compat/schema.rs`, `src/compat/contracts.rs`, and `src/compat/release.rs`.
  - Implement `CeDocsConfig`, `CeSolutionFrontmatter`, `CeSkillContract`, and `CeRelease` with unit tests.

- [x] **Unit 2 (~190 LOC):** Refactor Existing Commands to Use `src/compat/`
  - Refactor `src/commands/doc.rs` and `src/commands/workflow.rs` to delegate `docs_root` and frontmatter parsing to `src/compat/`.
  - Verify all existing unit tests in `doc` and `workflow` pass cleanly.

- [x] **Unit 3 (~200 LOC):** Implement Fleet State & Harness Driver Trait (`src/fleet/`)
  - Add `FleetState` (`pinned_version`, `last_sync`) to `State` in `src/state/state.rs`.
  - Implement `FleetHarnessDriver` trait in `src/fleet/driver.rs`.
  - Implement native drivers for OpenCode, Claude Code, Pi, and Codex/Cursor.

- [x] **Unit 4 (~190 LOC):** Implement `ce-ai fleet` CLI Subcommand (`status`, `pin`, `sync`)
  - Add `Fleet` subcommand to `src/main.rs`.
  - Create `src/commands/fleet.rs` implementing `ce-ai fleet status`, `ce-ai fleet pin <version>`, and `ce-ai fleet sync [--dry-run]`.
  - Add CLI integration tests for `ce-ai fleet`.

- [x] **Unit 5: Code Simplification & Review (`ce-simplify-code`)**
  - Ensure documentation adheres strictly to Diátaxis and cognitive load rules (`docs-styling.md`).
  - Verify zero clippy or format warnings across the workspace.

- [x] **Unit 6: Structured Code Review & Review Receipt**
  - Record workflow review receipt: `cargo run -- workflow review-receipt`.

- [x] **Unit 7: Knowledge Compounding & Vocabulary Capture (`ce-compound`)**
  - Accrete new domain concepts in `CONCEPTS.md`: *Fleet Version Governance Engine*, *Native Harness Driver Pattern*, and *Centralized Upstream Compatibility Boundary*.
  - Verify with `python3 scripts/validate-concepts.py CONCEPTS.md`.
  - Document solution learning in `docs/solutions/architecture/ce-compatibility-layer-and-fleet-governance.md`.
  - Verify `cargo run -- doc lint --strict`.

- [x] **Unit 8: Verification, SemVer bump, and PR Creation (`ce-commit-push-pr`)**
  - Bump MINOR version to `1.76.0` in `Cargo.toml` and record changes in `CHANGELOG.md`.
  - Run full test suite: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
  - Push branch and open PR with upfront empirical evidence.
