# Proposal: Installer Download Resilience

## Problem Statement

Both installers (`scripts/install.ps1`, `scripts/install.sh`) resolve
`releases/latest` and download the asset **once**, with no retry or fallback.
During a release publication window (assets uploading sequentially), the API
lookup and/or download return 404 and the installer dies. This produced CI
failure #298 ("Windows PowerShell Installer Gate", HTTP 404) even though the
PR content was correct — the repo now cuts releases concurrently from parallel
sessions, making the race routine rather than exceptional.

## In Scope

- Retry loop (3 attempts, linear backoff) around resolve+download in both installers.
- Fallback scan of the 5 most recent releases when `latest` has no matching asset.
- Clear final error listing attempted URLs.

## Out of Scope

- Checksum/signature verification of assets (separate concern).
- Changes to release workflow ordering.

## Risk

Low: additive retry/fallback logic; default success path unchanged.

## Success Criteria

- A 404 during a publication window no longer fails installation if assets
  become available within the retry window or exist in a recent prior release.
- CI Windows PowerShell Installer Gate passes on PRs opened mid-release.
