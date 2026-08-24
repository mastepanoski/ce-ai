# Proposal: Upgrade Provenance Binding & Honest Sync Verification

Resolves Issue #161 (P1 — supply-chain provenance and honest verification output).

## Problem Statement

Three related trust failures in the upgrade/sync pipeline:

1. **`upgrade --to <tag>` does not fetch or select that tag.** It reuses the single
   tarball digest cached under `managed_asset_digest["tarball"]` with no tag
   association, then records the caller-provided tag into
   `install-manifest.json` and `state.json` as if that artifact were the
   requested release. A cached `vY` archive can be relabelled as `vX`.
2. **No archive integrity gate on reuse.** A tampered or corrupted cached
   tarball is extracted and synced without comparing its actual SHA256 against
   any recorded provenance.
3. **`sync` prints fabricated verification output.** Every detected harness is
   reported as `synced & verified (... SHA256 integrity match)` and
   `reconciliation status: 100% Verified (0 drift)` although only the shared
   OpenCode managed directory was reconciled; no per-harness check runs and no
   failure can influence the exit status.

Additionally, `upgrade --harness <t>` (`-t`) and `upgrade --force` (`-f`) are
accepted silently but ignored by the implementation.

## Scope Boundaries

- **In Scope**:
  - New atomic provenance record `{tag, url, archive_sha256, extraction_path}`
    persisted together with the tarball digest in `state.json` via
    `write_atomic`.
  - `upgrade --to <tag>` binds to the exact matching provenance entry: tag
    mismatch fails with a precise usage error; archive bytes are re-hashed and
    compared before use; corruption fails closed (`Verification`, exit 6).
  - Default/latest-release path records full provenance at fetch time.
  - Remove unused `--harness`/`--force` flags from `ce-ai upgrade`; clap then
    rejects them as unknown arguments (usage error, exit 2).
  - Post-sync verification that only reports checks actually executed:
    OpenCode managed surface always hash-verified after apply; harness skill
    copies verified when performed; registration-only harnesses explicitly
    reported as unverified. Failures produce `CeError::Verification`
    (exit code 6).
- **Out of Scope**:
  - Fetching arbitrary remote tags for `--to` when the cache does not match
    (fails instead; fetching is the default no-flag path's job).
  - Per-harness managed-file manifests for every adapter (#155 territory).
  - The GitHub API 403 fallback hardening (Issue #202, covered by
    `multi-harness-reconciliation`).

## Risk Evaluation

- `state.json` gains an optional field with `#[serde(default)]`; older state
  files keep loading unchanged.
- Removing never-implemented flags changes CLI surface: scripts passing
  `--harness`/`--force` to `upgrade` will now fail fast with exit 2 instead of
  being silently ignored — this is the intended fix, not a regression.
- Verification failures now change exit status from 0 to 6; automation relying
  on the previous dishonest "always green" sync output must treat exit 6 as
  drift signal (documented in CHANGELOG).

## Success Criteria

- Requested-tag mismatch: upgrading `--to vX` with a cache entry for `vY`
  fails; state/manifest never record `vX` for a `vY` artifact.
- Cache-corruption: tampered archive SHA fails closed with a clear error
  naming expected vs actual digest.
- Sync output names exactly what was verified per harness; anything unverified
  is labelled as such; failures set exit code 6.
- `--harness` / `--force` on `upgrade` produce usage errors.
