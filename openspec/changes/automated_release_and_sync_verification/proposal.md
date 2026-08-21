# OpenSpec Proposal: Automated Release Workflow & Sync Verification Matrix

## Problem Statement
Provide deterministic, automated release packaging on `main` merges and itemized file-hash verification for `ce-ai sync`.

## Proposed Changes
1. **GitHub Release Automation Workflow**: Add `.github/workflows/release.yml` to automatically bump patch versions on `main`, build binaries across Linux/macOS/Windows, and publish GitHub Releases.
2. **Itemized Sync Verification Audit**: Update `ce-ai sync` and `ce-ai status` to output a per-file SHA256 integrity verification table across all active harnesses.
