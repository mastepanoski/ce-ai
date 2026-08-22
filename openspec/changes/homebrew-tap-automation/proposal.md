# Proposal: Automated Homebrew Tap Updates

## Problem

`ce-ai` is distributed via the Homebrew tap `mastepanoski/ce-ai`
(repository `mastepanoski/homebrew-ce-ai`). Today, every release requires a
manual, error-prone update of `Formula/ce-ai.rb` in the tap: bumping the
version, rewriting four download URLs, and recomputing four SHA256 checksums.
Forgetting this step leaves Homebrew users pinned to stale versions.

## In Scope

- A self-updating workflow in the tap repository
  (`homebrew-ce-ai/.github/workflows/self-update.yml`) that polls ce-ai
  releases on a schedule and updates the formula using the tap's own
  `GITHUB_TOKEN` — no cross-repo PAT required.
- Pinning the v1.0.8 SHA256 checksums in this repo's local
  `Formula/ce-ai.rb` (source of truth for the template).

## Out of Scope

- Submitting the formula to `homebrew/core`.
- Windows assets in the formula (Homebrew does not install `.zip` CLI
  binaries; Windows users use `install.ps1`).
- Changes to the existing build/release pipeline (`release.yml`).

## Risks

- Update latency: the tap polls every 30 minutes, so a new release can take
  up to ~30 minutes to reach Homebrew users. Accepted tradeoff vs. minting
  a cross-repo PAT.
- Checksum mismatch or missing asset fails the workflow before any push,
  so a broken formula can never reach users.

## Success Criteria

- Publishing a release automatically updates the tap formula within 30
  minutes, with zero secrets configured in either repository.
- `brew upgrade mastepanoski/ce-ai/ce-ai` picks up the new version.
- No manual SHA256 computation is ever needed again.
