---
title: "Multi-Harness Propagation, Upgrade Resolution, Automated Releases & Sync Verification"
date: "2026-08-21"
category: "multi-harness-operations"
module: "harness"
problem_type: "architecture"
components:
  - "harness"
  - "sync"
  - "upgrade"
  - "install"
  - "status"
  - "tui"
  - "ci"
applies_when: "Managing multiple AI agent harnesses (OpenCode, Claude, Pi, Cursor, Copilot, Kimi, Antigravity), upgrading from local source, or verifying sync reconciliation integrity."
tags:
  - "multi-harness"
  - "sync"
  - "upgrade"
  - "verification"
  - "github-actions"
---

# Solution: Multi-Harness Propagation, Upgrade Resolution, Automated Releases & Sync Verification Matrix

## Context
`ce-ai` originally managed only OpenCode (`opencode.json`). As teams adopted multiple AI coding tools (Claude Code, Pi, Cursor, Copilot, Kimi, Antigravity/Gemini), several gaps emerged:
1. `install --harness all` and `ce-ai sync` hardcoded `opencode` as the only target config directory (`~/.config/opencode/opencode.json`), causing modifications to be omitted on other active host harnesses.
2. `ce-ai upgrade` skipped local source installations without converting them to GitHub release tags.
3. Patch releases were created manually rather than automatically on GitHub Actions on `main` merges.
4. Users had no deterministic audit trail to empirically verify that `ce-ai sync` reconciled 100% of managed files across all active harnesses without drift.

## Solution Architecture

### 1. Multi-Harness Host Probing (`src/harness/mod.rs`, `src/commands/status.rs`, `src/tui.rs`)
- Added `is_ce_installed` and `detect_ce_installed_harnesses` to `HarnessKind`.
- Updated `ce-ai status` and TUI `Status & Harnesses` tab to probe host-detected agent harnesses alongside `state.json` entries.

### 2. Upgrade Resolution & Local Source Conversion (`src/commands/upgrade.rs`)
- Removed the blocking early-return guard for `source: local`.
- `ce-ai upgrade` (and TUI `Upgrade Release`) fetches the latest GitHub release tag, extracts the tarball, updates managed skills/loaders across all target harnesses, and updates `state.json` from `kind: local` to `kind: github-release`.

### 3. Multi-Harness Target Propagation (`src/commands/install.rs`, `src/commands/sync.rs`)
- **`install::run`**: Iterates over `target_harnesses` when `--harness all` is passed, ensuring plugin entries and skill paths are merged into every target harness configuration file (`claude.json`, `config.json`, `.cursorrules`, `antigravity.json`, `kimi.json`, etc.).
- **`sync_with`**: Probes host-detected CE installations (`detect_ce_installed_harnesses`) alongside registered entries. The OpenCode arm applies `ensure_plugin_and_skills` (plugin entry + skills paths) plus companion registration; every table-driven harness arm routes through `registration_spec` → `register_companions` and re-harvests its `install-manifest.json` SHA256 entries from the on-disk managed tree (v1.48.0), populating `state.installed_harnesses` for all active harnesses.

### 4. Deterministic Sync Verification Matrix (`src/commands/sync.rs`)
- `ce-ai sync` outputs a per-harness SHA256 integrity verification table
  (real output shape; registration-only harnesses such as claude/agy/kimi
  default to `registered` because adoption — not file copies — delivers
  their skills):
  ```text
  == [Sync Verification Matrix] ==
  version: v1.48.0
  source: github-release
    ✓ opencode: verified — 412/412 managed files match SHA256
    ○ claude: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
    ○ agy: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
    ○ kimi: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)
    ○ cursor: registered — config registration only — no managed assets to hash-verify
  reconciliation status: 1 verified, 4 registered (nothing to verify), 0 failed
  ```
- Post-v1.48.0: per-harness manifests are verified independently of the
  sync matrix. `ce-ai doctor` diffs the manifests of every
  `DRIFT_PROBE_HARNESSES` harness (claude, kimi) against their managed
  trees (`non_opencode_diff_findings`, `src/commands/doctor.rs`), and
  both `ce-ai status` and `ce-ai workflow resume` probe the same
  harness-manifest set (`DRIFT_PROBE_HARNESSES` + opencode,
  `src/commands/workflow.rs`), so hash integrity for registration
  harnesses is certified outside the matrix as well.

### 5. Automated GitHub Release Workflow (`.github/workflows/release.yml`)
- Triggers on `push.branches: [main]` and `tags: ['v*']`.
- Cross-compiles `ce-ai` binaries for Linux (`x86_64`), macOS (`x86_64`, `aarch64`), and Windows (`x86_64`), automatically publishing GitHub Releases on `main` pushes.

## Empirical Verification
- **Unit & Integration Tests**: 57 unit tests + 22 integration tests passing cleanly (`cargo test`).
- **Linter & Formatting**: 0 Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`), 100% clean formatting (`cargo fmt`).
- **Containerized E2E Gate**: `make e2e` executing containerized Linux E2E test runner passed 100% green.
- **GitHub PRs Merged**: PR #24, PR #25, PR #26, PR #27, PR #29, PR #30, PR #31, PR #32, PR #33.
