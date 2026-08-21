# OpenSpec Specification: Release & Verification

### AR-1: Automated GitHub Release Workflow
- **WHEN** commits land on `main`,
- **THEN** `.github/workflows/release.yml` MUST automatically generate a release tag, build release binaries for Linux/macOS/Windows, and publish a GitHub Release.

### SV-1: Itemized Sync Verification Matrix
- **WHEN** `ce-ai sync` runs (or is invoked from TUI),
- **THEN** `ce-ai` MUST output a per-harness, per-file verification table detailing SHA256 integrity checks and zero-drift confirmation.
