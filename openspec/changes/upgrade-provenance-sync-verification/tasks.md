# Tasks: Upgrade Provenance & Honest Sync Verification

TDD order: failing test first per task, then implementation.

## 1. Error surface
- [ ] 1.1 Add `CeError::Verification(String)` (Display + exit code 6) with mapping test; run `cargo test error`.

## 2. State schema
- [ ] 2.1 Failing tests: `ReleaseProvenance` round-trips through `state.json`; legacy file without the field loads.
- [ ] 2.2 Implement `ReleaseProvenance` + optional `State.release_provenance` (serde default).

## 3. Cache & provenance recording
- [ ] 3.1 Refactor `Cache::cache_tarball` to return `(path, hex)` without state writes; update install.rs caller.
- [ ] 3.2 Implement `record_tarball_provenance` (single atomic write of digest + provenance); unit test asserts both keys land in one save.

## 4. `--to` binding & integrity gate
- [ ] 4.1 Failing tests: tag mismatch → `Usage`, state unchanged; tampered archive → `Verification` fail-closed; matching tag resolves.
- [ ] 4.2 Implement `cached_tarball_for` replacing `cached_tarball`; wire into `run`.

## 5. Dead flags
- [ ] 5.1 Test: clap rejects `--harness`/`--force` on upgrade; delete the fields.

## 6. Honest sync verification
- [ ] 6.1 Unit tests for `verify_tree_against` (match/mismatch/missing).
- [ ] 6.2 Implement post-apply verification matrix in `sync_with` (opencode managed surface always; copied skill trees when performed; others labelled unverified) and failure → `Verification`.
- [ ] 6.3 Integration-style test: drift after sync produces exit-6 error and honest output data.

## 7. Verification gates
- [ ] 7.1 `cargo fmt --check`
- [ ] 7.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 7.3 `cargo test`
- [ ] 7.4 `make e2e`

## 8. Shipping
- [ ] 8.1 Update `CHANGELOG.md`, bump `Cargo.toml` + `Formula/ce-ai.rb` (patch), check README mentions of removed flags.
- [ ] 8.2 Conventional commit, push, `gh pr create` referencing #161, watch CI green.
