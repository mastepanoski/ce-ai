# Technical Implementation Plan: Automated Release Workflow & Sync Verification Matrix

**Date**: 2026-08-21  
**Origin**: `docs/brainstorms/2026-08-21-automated-release-and-sync-verification-requirements.md`  
**OpenSpec Specifications**: `openspec/changes/automated_release_and_sync_verification/`  

---

## 1. Scope & Architecture

### Unit 1: GitHub Release Automation Workflow (`.github/workflows/release.yml`)
- Trigger: `push` on `main`.
- Automates SemVer patch version tags (`vX.Y.Z`), builds cross-platform binaries (Linux x86_64, macOS x86_64/arm64, Windows x86_64), and publishes GitHub Releases automatically via `gh release create`.

### Unit 2: Deterministic Sync Verification Matrix (`src/commands/sync.rs`)
- Enhances `sync_with` to compute SHA256 integrity matches for every managed file across every active harness target (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `kimi`, `agy`, etc.).
- Prints itemized verification matrix in CLI and TUI:
  ```
  == [Sync Verification Matrix] ==
  harness: opencode
    ✓ plugins/compound-engineering.js [SHA256 Match: e3b0c442...]
    ✓ skills/ce-brainstorm/SKILL.md [SHA256 Match: a1b2c3d4...]
  harness: claude
    ✓ plugins/compound-engineering.js [SHA256 Match: e3b0c442...]
  reconciliation status: 100% Verified (0 files drifted, all active harnesses synced)
  ```

### Unit 3: CLI Integration Tests (`tests/cli.rs`)
- Verify sync verification matrix output in integration test suite.
