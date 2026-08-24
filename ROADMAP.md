# `ce-ai` — Product & Release Roadmap

This document outlines the version milestones, feature criteria, and release pipeline strategy leading to **Version 1.0.0 (Production Stable)**.

---

## 🎯 Vision & Criteria for `v1.0.0` Production Release

`ce-ai` reaches **`v1.0.0`** when it achieves:
1. **Multi-Harness Stability**: Full support and verification across all 10 native AI coding agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`) plus custom mode.
2. **Zero-Data-Loss Backup & Recovery**: Complete backup listing, point-in-time snapshot recovery, and clean uninstallation guarantees.
3. **Automated Release Pipeline**: Automated GitHub Actions CI/CD pipeline compiling and publishing signed release binaries (Linux, macOS, Windows) on version tags (`v*`).
4. **Automated Update Checking**: Proactive update notifications in CLI and TUI for both `ce-ai` and `compound-engineering-plugin`.
5. **Stable API & CLI Contract**: Frozen CLI flag structures, exit code specifications, and schema definitions.

---

## 🗺️ Version Milestones to `v1.0.0`

```
  v0.7.0 (Released) ──► v0.8.0 (Released) ──► v0.9.0 (Released)
                                                     │
  v1.0.0 (Released) ◄────────────────────────────────┘
```

### ✅ `v0.3.0` & `v0.4.0` (Completed) — Multi-Harness Architecture & TUI
- [x] Multi-harness engine supporting 12 AI coding agent targets.
- [x] Host environment auto-probing (`HarnessKind::detect_installed_harnesses`) for `~/.claude`, `~/.pi`, `~/.kimi-code`, `~/.gemini/antigravity-cli`, `~/.codex`.
- [x] Interactive harness selection (`< [ target ] >`) in TUI dashboard.
- [x] Pre-commit security gate and green CI matrix (Linux, macOS, Windows, Docker E2E).
- [x] Mandatory Pull Request governance workflow (`AGENTS.md`).

---

### ✅ `v0.5.0` (Released) — Workspace Scope, Companion Tools & Workflow FSM ([#5](https://github.com/mastepanoski/ce-ai/issues/5), [#7](https://github.com/mastepanoski/ce-ai/issues/7), [#9](https://github.com/mastepanoski/ce-ai/issues/9), [#10](https://github.com/mastepanoski/ce-ai/issues/10))
- [x] **Workspace Scope Isolation**: Added `--scope workspace|global` flag in `ce-ai install` resolving repository root via `git rev-parse`.
- [x] **Companion Tools Manager**: Added `ce-ai tools status` and `ce-ai tools install` for developer sidecars (`Engram`, `CodeGraph`, `Context7`, `RTK`).
- [x] **Workflow FSM & Recovery Engine**: Added `ce-ai workflow status`, `checkpoint`, and `resume` tracking 7-stage progress and context recovery.
- [x] **Backup Management & Point-in-Time Recovery**: `ce-ai backups list/restore` CLI and dedicated TUI panel (`MenuTab::Backups`).
- [x] **Automated Release Pipeline**: Added `.github/workflows/release.yml` for multi-platform cross-compilation.

---

### ✅ `v0.6.0` (Released) — Proactive Workflow Observability & Sync Watcher
- [x] **TUI Workflow Dashboard**: Dedicated `🎮 Workflow (FSM)` panel in `ce-ai tui` visualizing 7-stage Flywheel status and progress checkpoints.
- [x] **Extended Companion Diagnostics**: Non-fatal health probes in `ce-ai doctor` for Engram SQLite DB, CodeGraph index, and RTK binary PATH.
- [x] **Real-Time Sync Watcher**: `ce-ai sync --watch` for real-time drift monitoring and automatic SHA256 re-syncing.
- [x] **Teacher-Style Documentation**: Explaining cockpit instrument panels and autopilot guardrails in user guides.

---

### ✅ `v0.7.0` (Released) — Workspace Overrides & Multi-Harness Uninstall Parity ([#64](https://github.com/mastepanoski/ce-ai/issues/64))
- [x] **Workspace Configuration Overrides (`.ce-ai.json`)**: Local repository overrides with key-level precedence resolution over global `~/.config/ce-ai/state.json`.
- [x] **Complete Multi-Harness Uninstall Parity**: Extended `ce-ai uninstall` with `--harness <name|all>`, `--all`, and `--yes` / `-y` flags for complete removal of managed loaders and skills across all installed harnesses cleanly.
- [x] **Teacher-Style Documentation**: Updated masterclass user guide with local cockpit settings vs master flight plan analogies.

---

### ✅ `v0.8.0` (Released) — Universal Installer & Multi-Arch Release Matrix ([#2](https://github.com/mastepanoski/ce-ai/issues/2), [#3](https://github.com/mastepanoski/ce-ai/issues/3), [#28](https://github.com/mastepanoski/ce-ai/issues/28))
- [x] **Multi-Platform Release Pipeline (`.github/workflows/release.yml`)**: Automated 6-target build matrix compiling Linux/macOS/Windows release binaries for `x86_64` and `ARM64`.
- [x] **Universal One-Line Installer Script**: Cross-platform POSIX (`scripts/install.sh`) and PowerShell (`scripts/install.ps1`) scripts for zero-dependency installation.
- [x] **Package Manager Homebrew Formula**: Formula template `Formula/ce-ai.rb`.

---

### ✅ `v0.9.0` (Released) — Hardening, Performance & Security Audit
- [x] **ISO 27001 / ISO 27002 Security Audit Suite**: Dedicated threat matrix test suite `tests/security.rs` verifying path traversal payload rejection and atomic write integrity.
- [x] **Sub-50ms High-Performance Benchmarks**: Benchmark suite `benches/benchmarks.rs` verifying sub-50ms execution bounds for state resolution, workspace overrides, and SHA256 integrity hash calculation.
- [x] **100% Core Test Coverage & Library API Export**: Added `src/lib.rs` exporting core modules cleanly.

---

### 💎 `v1.0.0` (Released) — Production Release (Stable API Freeze) ([#72](https://github.com/mastepanoski/ce-ai/issues/72), [#75](https://github.com/mastepanoski/ce-ai/issues/75), [#76](https://github.com/mastepanoski/ce-ai/issues/76))
- [x] **Frozen CLI Contract & Configuration Schemas**: Stable subcommand flags and locked JSON schemas (`state.json`, `.ce-ai.json`, `opencode.json`).
- [x] **TUI Modal Text Wrapping Fix (Issue #72)**: Ratatui Paragraph line wrapping (`Wrap { trim: false }`) in `MenuTab::Sync` and `MenuTab::Doctor` result modals.
- [x] **TUI Direct Stage Invocation (Issue #76)**: Direct stage transition key shortcuts (`[1-7]`) inside `MenuTab::Workflow` panel for rapid Flywheel phase switching.
- [x] **Bug Report Template (Issue #75)**: GitHub Issue Form `.github/ISSUE_TEMPLATE/bug_report.yml`.
- [x] **Homebrew Installation Guidance**: Documented Homebrew tap and formula installation commands in `README.md`.

---

## 📦 Release Generation Strategy & Cadence

| Release Type | Trigger / Cadence | Naming / Tag | Purpose |
| :--- | :--- | :--- | :--- |
| **Patch Release** | Bug fixes & security patches (as needed) | `v0.4.x` | Immediate fix shipping via PR workflow. |
| **Minor Release** | Feature milestone completion (2-3 weeks) | `v0.5.0`, `v0.6.0`, etc. | Key capability additions (e.g. Backups, Update Checker). |
| **Major Release (`v1.0.0`)** | Production gate criteria fully met | `v1.0.0` | Production stable release with guaranteed backwards compatibility. |

---

## 📜 How Releases Are Generated (Step-by-Step)

When a release milestone (e.g. `v0.5.0`) is ready:

1. **Create Release Branch**:
   ```bash
   git checkout -b release/v0.5.0
   ```
2. **Update Version & Changelog**:
   - Update `version = "0.5.0"` in `Cargo.toml`.
   - Update `CHANGELOG.md` with release notes under `## [0.5.0] - YYYY-MM-DD`.
3. **Open Pull Request**:
   - Submit PR to `main` (`gh pr create`).
   - Wait for 100% green CI build, tests, Docker E2E gate, and security audit.
4. **Merge PR & Create Release Tag**:
   ```bash
   git checkout main && git pull
   git tag -a v0.5.0 -m "Release v0.5.0"
   git push origin v0.5.0
   ```
5. **Automated CI/CD Release Compilation**:
   - GitHub Actions compiles binaries for Linux, macOS, and Windows.
   - Publishes GitHub Release with checksums and release notes.
