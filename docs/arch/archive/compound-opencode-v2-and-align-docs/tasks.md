# Tasks: Compound Capture & Doc Styling Alignment

- [x] **Unit 1 (~120 LOC):** Document OpenCode V2 bugfix solution in `docs/solutions/bugfixes/opencode-v2-dual-loader.md`
  - Create solution artifact with YAML frontmatter (`module: opencode`, `problem_type: bugfix`, `tags: [opencode, plugin-loader, v2-migration, esm, native-commands]`).
  - Document symptoms, dual root causes (config key rename + ESM default export requirement), dual export solution architecture, native command materialization, and local empirical verification.
  - *Verification*: `cargo run -- doc lint --strict`.

- [x] **Unit 2 (~40 LOC):** Monotonic concept accretion in `CONCEPTS.md`
  - Read `CONCEPTS.md` pre-mutation.
  - Surgically add `OpenCode Dual Plugin Loader`, `Native Command Materialization`, and `OpenCode Config Key Agnosticism`.
  - *Verification*: `cargo run -- doc lint --strict` confirming monotonic accretion without shrinkage.

- [x] **Unit 3 (~50 LOC):** Documentation alignment with `docs/references/docs-styling.md`
  - Refactor `README.md` to be strictly ≤ 100 lines and place Quick Path directly after Title/What & Why.
  - Normalize `docs/user-guide/project-adoption-guide.md` intent to singular `How-to`.
  - Add markdown links in `docs/references/docs-styling.md` for reference and explanation examples.
  - *Verification*: `wc -l README.md` (≤ 100 lines) and manual review against `docs-styling.md` checklist.

- [x] **Unit 4 (~20 LOC):** Simplification audit & review receipt
  - Run `ce-simplify-code` audit on `src/opencode/` to confirm zero redundant or hacky patterns.
  - Record code-review receipt with `cargo run -- workflow review-receipt`.
  - *Verification*: `cargo run -- workflow status` reflecting valid review receipt.

- [x] **Unit 5 (~10 LOC):** Verification, SemVer bump, and PR creation
  - Bump PATCH version in `Cargo.toml` (`1.74.1`) and add entry to `CHANGELOG.md`.
  - Run full test gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`, `cargo run -- doc lint --strict`.
  - Open PR with self-explaining description and empirical evidence.
