# Tasks: Close Determinism Leaks 1 & 2

Per-work-unit changed-line estimates target ~200 LOC (CONTRIBUTING.md §4).

## Work Unit 1 — Deterministic source resolution (~120 LOC)

- [ ] T1.1 Add failing unit test: `pinned_version_and_url(None)` returns
      `CeError::Usage`; `Some(tag)` returns tag + `tag_tarball_url`.
- [ ] T1.2 Implement `pinned_version_and_url` in `src/source/release.rs`.
- [ ] T1.3 Rewrite `resolve_latest_release` failure paths to
      `CeError::Network` / propagate parse errors; update module docs.
- [ ] T1.4 Replace fallback match arms in `upgrade.rs::run` and
      `install.rs` fetch path; remove now-unused imports and delete
      `main_tarball_url`.
- [ ] T1.5 Verify: `cargo test -D warnings-level clippy` subset passes for the
      touched crates.

## Work Unit 2 — Byte-stable skill resolution output (~60 LOC)

- [ ] T2.1 Add failing unit test `resolve_markdown_is_byte_stable` in
      `src/source/registry.rs`.
- [ ] T2.2 Remove `timestamp=` field and `Utc::now()` call from `resolve`;
      drop unused `chrono` import if orphaned.

## Work Unit 3 — Documentation & versioning (~150 LOC)

- [ ] T3.1 Update Step 1 of `docs/user-guide/sync-and-upgrade-mechanisms.md`
      with the fail-loudly contract.
- [ ] T3.2 Write `docs/user-guide/determinism-explained.md` (Explanation,
      Beginner): what determinism is, asset-layer guarantees, why LLM
      execution cannot be deterministic, environment-relative behaviors,
      compensating controls.
- [ ] T3.3 Add README documentation-map row; keep README ≤ 100 lines.
- [ ] T3.4 Bump `Cargo.toml` to 1.23.0; add CHANGELOG entry under `[1.23.0]`.

## Work Unit 4 — Verification gates (~0 LOC)

- [ ] T4.1 `cargo fmt --check`
- [ ] T4.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] T4.3 `cargo test`
- [ ] T4.4 `make e2e`
