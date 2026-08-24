# Task Breakdown: Grok Adapter Audit Refinements

- [x] Create OpenSpec contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`)
- [x] Run `ce-doc-review` panel on OpenSpec contract and address findings
- [x] Implementation:
  - [x] Add static `HARNESS_ENV_LOCK` mutex lock in `src/harness/mod.rs` and `src/harness/grok.rs` unit tests to synchronize `GROK_HOME` access
  - [x] Remove legacy `HarnessKind::Grok` mapping and update module docstring in `src/harness/generic_json.rs`
  - [x] Bump SemVer to `v1.13.2` in `Cargo.toml` and `Formula/ce-ai.rb`, and update `CHANGELOG.md`
- [x] Quality gates (`cargo fmt`, `cargo clippy`, `cargo test`)
- [x] Run `ce-code-review` panel
- [x] Document solution via `ce-compound`
- [x] Ship PR, wait for 100% green CI matrix, merge, tag `v1.13.2`, release
