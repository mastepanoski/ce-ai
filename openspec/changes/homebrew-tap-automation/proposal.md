# Proposal: Automated Homebrew Tap Updates

## Problem

`ce-ai` is distributed via the Homebrew tap `mastepanoski/ce-ai`
(repository `mastepanoski/homebrew-ce-ai`). Today, every release requires a
manual, error-prone update of `Formula/ce-ai.rb` in the tap: bumping the
version, rewriting four download URLs, and recomputing four SHA256 checksums.
Forgetting this step leaves Homebrew users pinned to stale versions.

## In Scope

- A GitHub Actions workflow (`.github/workflows/bump-homebrew.yml`) that runs
  when a release is published.
- The workflow downloads the four platform tarballs from the release,
  computes their SHA256 digests, renders `Formula/ce-ai.rb`, and pushes it to
  the tap repository.
- Manual re-run support via `workflow_dispatch` with an explicit tag input.

## Out of Scope

- Submitting the formula to `homebrew/core`.
- Windows assets in the formula (Homebrew does not install `.zip` CLI
  binaries; Windows users use `install.ps1`).
- Changes to the existing build/release pipeline (`release.yml`).

## Risks

- Cross-repo push requires a fine-grained PAT stored as the repository
  secret `TAP_TOKEN`. If missing, the workflow fails loudly instead of
  silently skipping.
- Checksum mismatch or missing asset fails the workflow before any push,
  so a broken formula can never reach users.

## Success Criteria

- Publishing a release automatically updates the tap formula within minutes.
- `brew upgrade mastepanoski/ce-ai/ce-ai` picks up the new version.
- No manual SHA256 computation is ever needed again.
