# Exploration: Automated Homebrew Tap Updates

## Options Evaluated

### Option 1: Manual tap maintenance (status quo)

Every release: download 4 assets, run `shasum -a 256`, edit
`Formula/ce-ai.rb`, push to the tap. Rejected: error-prone, easy to forget,
checksum mistakes ship broken formulas.

### Option 2: Submit to homebrew/core

Requires notability thresholds (30+ forks/watchers), human review, and a
long merge cycle. Rejected for now; can be revisited when ce-ai has
traction.

### Option 3 (chosen): Release-triggered workflow pushing to the tap

A dedicated workflow triggered by `release: published`:

1. Downloads the four platform tarballs from the published release.
2. Computes SHA256 digests.
3. Renders `Formula/ce-ai.rb` from a heredoc template with the version and
   digests interpolated.
4. Checks out `mastepanoski/homebrew-ce-ai`, commits the formula, pushes to
   `main`.

## Key Decisions

- **Trigger on `release: published`, not tag push**: `release.yml` builds
  binaries asynchronously after the tag lands; assets only exist once the
  release is published. Listening to the release event guarantees assets
  are present.
- **Cross-repo auth**: GitHub's default `GITHUB_TOKEN` is scoped to one
  repository. Pushing to `homebrew-ce-ai` requires a fine-grained PAT with
  `Contents: read/write` on the tap repo only, stored as secret `TAP_TOKEN`.
  Least privilege; no org-wide credentials.
- **Fail before write**: all downloads and checksum computation happen in
  the ce-ai workspace. The tap is touched only if every asset verified.
  This prevents publishing a partial or corrupt formula.
- **Idempotency**: re-running for an already-bumped tag produces an empty
  commit which git skips naturally (`git diff --cached --quiet`); the step
  tolerates this so manual re-runs never fail spuriously.

## Architectural Tradeoffs

- A heredoc template inside the workflow duplicates the formula shape that
  also lives in this repo's `Formula/ce-ai.rb`. Extracting it into a shared
  script adds indirection for a ~30-line file; duplication is accepted and
  the repo copy remains the human-readable source of truth.

## Verification Evidence

- The heredoc template was rendered locally with the v1.0.8 checksums and
  compared byte-for-byte against the formula already published and
  install-tested in `mastepanoski/homebrew-ce-ai`: files are identical.
