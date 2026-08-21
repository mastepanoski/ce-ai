# Automated Release Workflow & Deterministic Sync Verification Requirements

## 1. Problem Statement
1. **Manual Version Releases**: Releases and patch version bumps in `Cargo.toml` were not automatically created when commits merged to `main`. A deterministic GitHub Release workflow is needed.
2. **Lack of Sync Verification**: Users have no visual or audit mechanism to verify whether `ce-ai sync` actually reconciled files across all active harnesses or if drift remains.

## 2. Key Requirements

### R-1: Automated GitHub Release & SemVer Patch Bumper
- **WHEN** commits or PRs are merged into `main`,
- **THEN** a GitHub Actions release workflow MUST deterministically:
  1. Increment `Cargo.toml` patch version (e.g. `v0.4.0` -> `v0.4.1`).
  2. Create a Git tag `vX.Y.Z` and push to origin.
  3. Publish a formal GitHub Release with release notes and cross-platform pre-compiled binaries (`linux-x86_64`, `macos-x86_64`, `macos-arm64`, `windows-x86_64`).

### R-2: Deterministic Sync Verification & Itemized Audit Trail
- **WHEN** `ce-ai sync` or `ce-ai status --verify` is executed,
- **THEN** `ce-ai` MUST output a deterministic per-file verification matrix showing:
  - File path & target harness name (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `kimi`, `agy`, etc.)
  - SHA256 integrity match (`MATCH`, `RESTORED`, `CREATED`, `REMOVED`)
  - Summary of total reconciled files and zero-drift confirmation.

## 3. Success Criteria
- Merging to `main` triggers automated GitHub Release generation and binary release packaging.
- Running `ce-ai sync` outputs explicit per-harness file verification results so users can empirically confirm 100% sync integrity.
