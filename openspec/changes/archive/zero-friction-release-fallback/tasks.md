# Tasks: Zero-Friction Release Resolution with Web Redirect Fallback

Work units carry per-unit changed-line estimates (~200 LOC target) so the PR-level forecast is derivable (CONTRIBUTING.md §4). Total forecast: ~90 lines.

- [x] **Task 1: Web Redirect & Atom Feed Resolver Implementation** (~50 LOC)
  - [x] Implement `extract_tag_from_redirect_url` and `extract_latest_tag_from_atom_feed` in `src/source/release.rs`.
  - [x] Implement `resolve_latest_release_fallback(client)` and wire it into `resolve_latest_release()`.
  - [x] Add unit tests in `src/source/tests/release.rs` for URL/Atom feed tag extraction.
  - [x] Verification: `cargo test source::tests::release`

- [x] **Task 2: Doctor & UX Updates** (~15 LOC)
  - [x] Update `src/commands/doctor.rs` messaging to inform users that frictionless fallback is active when unauthenticated.
  - [x] Verification: `cargo test`

- [x] **Task 3: Version Bump & CHANGELOG** (~10 LOC)
  - [x] Bump version from `1.29.1` to `1.29.2` in `Cargo.toml`.
  - [x] Update `CHANGELOG.md` with Keep a Changelog format.
  - [x] Verification: `cargo check`

- [x] **Task 4: Full Quality Gates & E2E Verification** (~0 LOC)
  - [x] `cargo fmt --check`
  - [x] `cargo clippy --all-targets --all-features -- -D warnings`
  - [x] `cargo test`
  - [x] Live execution test of `ce-ai upgrade` without token (unauthenticated).
