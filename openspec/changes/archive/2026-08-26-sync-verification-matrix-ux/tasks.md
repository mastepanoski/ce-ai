# Tasks: Sync Verification Matrix UX Clarity

Work-unit LOC estimates follow CONTRIBUTING.md §4 (~200 LOC target per unit;
rescopes may only narrow).

## Work Unit 1 — Matrix rendering functions + pinned wording (~150 LOC)

- [x] **T1.1 (RED)** Add unit tests to `src/commands/sync.rs` `mod tests`
      pinning: registered line wording (SVX-1), verified/FAILED wordings and
      indented detail lines (SVX-2), reconciliation line with
      `registered (nothing to verify)` and no `unverified` substring (SVX-1),
      and guidance-note content incl. `install --harness` and scope boundary
      (SVX-3). Tests reference `matrix_line`, `failed_detail_lines`,
      `reconciliation_line`, `guidance_note_lines` — compile failure = RED.
- [x] **T1.2 (GREEN)** Implement the four pure functions per design.md; add
      the new static reason strings to the four `NotVerified` construction
      sites; rewire `run()`'s matrix printing to use them; print the note
      when `unverified > 0`.
- [x] **T1.3** Verify: `cargo test` green, `cargo fmt --check` clean,
      `cargo clippy --all-targets --all-features -- -D warnings` clean.

## Work Unit 2 — Docs + version + changelog (~80 LOC)

- [x] **T2.1** Update `docs/user-guide/sync-and-upgrade-mechanisms.md` Step 6:
      new sample output, "Verification states" subsection, "How ce-ai manages
      each harness" table with adoption command and scope boundary (SVX-4).
- [x] **T2.2** Bump `Cargo.toml` to `1.22.4`; add `CHANGELOG.md` `[1.22.4]`
      `### Changed` entry describing the matrix wording and guidance note.
- [x] **T2.3** Final gate before PR: full `cargo test`, fmt, clippy; run
      `ce-ai sync` against the local install and eyeball the new matrix.
