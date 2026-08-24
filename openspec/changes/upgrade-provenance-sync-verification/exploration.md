# Exploration: Upgrade Provenance & Honest Sync Verification

## Current Behavior (evidence)

- `src/commands/upgrade.rs:78` `cached_tarball()` reads
  `managed_asset_digest["tarball"]` — one digest, zero tag association. The
  requested `--to` tag is passed straight into `sync_from_extracted(..., tag,
  tag)` and recorded in manifest + state regardless of which artifact is on
  disk.
- `src/source/cache.rs:26` `cache_tarball()` writes the tarball file atomically
  and separately persists only the digest into state.
- `src/commands/sync.rs:345-362` prints a hardcoded verification matrix:
  per-harness `✓ synced & verified (N files, SHA256 integrity match)` for every
  entry of the active-harness list and `reconciliation status: 100% Verified
  (0 drift)`. No hash comparison runs against harness surfaces; the OpenCode
  managed dir diff (`state::diff`) drives copy/restore/remove but its outcome
  never feeds this output.
- `src/commands/upgrade.rs:25-30` declares `harness: String` (with
  `default_value = "all"`) and `force: bool`; neither is read by `run()`.
- `src/error.rs` has no `Verification` variant; AGENTS.md already reserves exit
  code 6 for verification failures.

## Options Evaluated

### Provenance storage

1. **Extend `install-manifest.json.source`** — already holds `{kind, tag,
   tree}` but the manifest is rewritten on every sync and lives under the
   OpenCode managed dir, not the global state surface. Rejected as primary;
   manifest keeps mirroring version/source as today.
2. **New `State.release_provenance: Option<ReleaseProvenance>`** written in the
   same atomic save that records the digest. ✔ Selected: single source of
   truth, serde-default backward compatible, satisfies "persist atomically
   together".

### Where to write provenance

`cache_tarball()` runs before extraction, so `extraction_path` is unknown
there. Writing twice would violate the atomic-together requirement. ✔ Record
once after `extract_to_source()` via a dedicated helper that sets **both**
`managed_asset_digest["tarball"]` and `release_provenance` in one
`State::save()` (temp+rename).

### `--to vX` vs cache holding vY

1. Fetch the requested tag from GitHub when mismatched — network dependency
   contradicts the flag's documented offline/test purpose. Rejected.
2. Fail with a precise error directing to the fetch path. ✔ Selected; matches
   acceptance criteria ("fails or fetches").

### Integrity check timing

Re-hash cached archive bytes at `--to` resolution time (before extraction).
Extraction is already traversal-safe (`extract_safe`); hashing first prevents
spending effort on corrupted archives and gives fail-closed semantics.

### Sync verification honesty

1. Verify every harness's full config surface — requires per-adapter manifests
   (#155). Out of scope.
2. Verify what sync actually wrote: OpenCode managed dir post-apply hash
   comparison (always), plus skill-tree copies performed for claude/codex/
   copilot/grok; registration-only adapters reported explicitly as unverified.
   ✔ Selected — output generated only from executed checks.
3. Exit status: any verified-surface failure returns
   `CeError::Verification(msg)` → exit 6.

## Architectural Tradeoffs

- Removing `--harness` re-adds friction for users who assumed it worked; but
  silent acceptance is a trust bug. clap's native unknown-arg error already
  exits with code 2, matching the `Usage` contract.
- `ReleaseProvenance.extraction_path` may point at a dry-run temp tree; in that
  case it is still recorded (mirrors today's manifest behavior) and the next
  real upgrade overwrites it.
