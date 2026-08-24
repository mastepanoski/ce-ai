# Exploration & Design: Adoption v2 Test Hardening & Stale-Block Doctor Signal

## Key design decision (doctor)

The current drift check is `text.contains(&expected_sha)` — the hash of the
**current** template body. A header-only version bump with an unchanged body
does not trip it, and that is correct: real stale blocks carry an **old body**
(missing SSOT section), so their hash differs. The new logic therefore only
classifies a mismatch as "stale" when the on-disk header declares
`v < BLOCK_VERSION`; any other mismatch stays generic drift.

Version parsing: scan lines for `BLOCK_BEGIN_MARKER` prefix, then parse
`v=<u32>` from the remainder; first parseable value wins; absent/unparseable →
generic drift (fail-closed to today's behavior).

## Test design notes

- LF variant mirrors the CRLF test byte-for-byte except line endings.
- Triangle test extracts header sha + body between markers, hashes the body
  with `sha2` (already a package dependency), and asserts equality with both
  the header and state.json's `block_sha256`.
- Malformed-block test asserts non-zero exit and that no partial managed
  content was appended.
- Doctor tests hand-write state.json entries via real `init-prj` runs, then
  mutate the file, asserting stdout contains the targeted message.

## Files

- Modify: `src/commands/doctor.rs`, `src/state/state.rs` (`AdoptionTier::as_str` helper)
- Test: `tests/cli.rs`
