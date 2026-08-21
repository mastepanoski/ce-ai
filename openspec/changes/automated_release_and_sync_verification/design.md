# OpenSpec Design: Release & Verification Architecture

## Release Workflow (`.github/workflows/release.yml`)
- Trigger: `push` on `main`.
- Jobs:
  - `bump-and-tag`: Reads `Cargo.toml`, increments patch version if new commits landed, tags `vX.Y.Z`, and pushes tag.
  - `build-binaries`: Cross-compiles `ce-ai` binaries for Linux (`x86_64`), macOS (`x86_64`, `aarch64`), and Windows (`x86_64`).
  - `publish-release`: Uses `gh release create` to upload release binaries and release notes.

## Sync Audit Matrix (`src/commands/sync.rs`)
- Add `SyncAuditReport` struct recording:
  - `harness_name: String`
  - `reconciled_files: Vec<(String, String, String)>` (path, status, sha256)
- Print detailed verification table:
  ```
  == [Sync Verification Matrix] ==
  harness: opencode
    ✓ plugins/compound-engineering.js (SHA256 Match)
    ✓ skills/ce-brainstorm/SKILL.md (SHA256 Match)
  harness: claude
    ✓ plugins/compound-engineering.js (SHA256 Match)
  sync status: 100% Verified (0 drift)
  ```
