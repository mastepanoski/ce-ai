# Changelog

All notable changes to `ce-ai` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [1.0.2] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Fixed boolean download validation flag and eliminated `$LASTEXITCODE` null evaluation syntax error in PowerShell.

---

## [1.0.1] - 2026-08-21

### Fixed
- **Windows PowerShell Installer (`scripts/install.ps1`)**: Added GitHub REST API resolution to retrieve direct release asset URLs, `$LASTEXITCODE` validation for native `curl.exe`, and TLS 1.2 fallback download verification.
- **CI Pipeline (`.github/workflows/ci.yml`)**: Added dedicated `windows-installation-gate` job running on `windows-latest` runners.

---

## [1.0.0] - 2026-08-21

### 💎 Production Stable Release
- **CLI Contract & Schema Freeze**: Frozen CLI subcommands (`install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, `doctor`, `workflow`, `tools`, `backups`, `tui`) and state schemas (`state.json`, `.ce-ai.json`, `opencode.json`) with strict backwards-compatibility guarantees.
- **TUI Modal Text Wrapping Fix (Issue #72)**: Ratatui Paragraph line wrapping (`Wrap { trim: false }`) in `MenuTab::Sync` and `MenuTab::Doctor` result modals preventing mid-word text breakage.
- **TUI Direct Stage Invocation (Issue #76)**: Direct stage transition key shortcuts (`[1-7]`) inside `MenuTab::Workflow` panel for rapid Flywheel phase switching.
- **Bug Report Template (Issue #75)**: GitHub Issue Form `.github/ISSUE_TEMPLATE/bug_report.yml`.
- **Homebrew Documentation**: Official installation commands for Homebrew tap and formula in `README.md`.

---

## [0.9.0] - 2026-08-21

### Added
- **ISO 27001 / ISO 27002 Security Audit Test Suite (`tests/security.rs`)**: Path traversal payload rejection, atomic state write tempfile cleanup, and corrupted JSON state recovery.
- **High-Performance Benchmarks (`benches/benchmarks.rs`)**: Verified sub-50ms execution bounds for state resolution, workspace overrides merging, and SHA256 integrity hash calculation.
- **Security Policy Update**: Updated `SECURITY.md` supported versions matrix.

---

## [0.8.0] - 2026-08-21

### Added
- **Multi-Platform Native Release Pipeline (Issue #28)**: GitHub Actions CI matrix cross-compiling release binaries for Linux (`x86_64`, `ARM64`), macOS (`Intel`, `Apple Silicon`), and Windows (`x86_64`, `ARM64`).
- **Universal One-Line Installer Script (Issue #3)**: Cross-platform POSIX (`scripts/install.sh`) and PowerShell (`scripts/install.ps1`) installer scripts for zero-dependency binary downloads.
- **Homebrew Formula Specification (Issue #2)**: Package manager formula template (`Formula/ce-ai.rb`).

---

## [0.7.0] - 2026-08-21

### Added
- **Workspace Configuration Overrides (`.ce-ai.json`)**: Added repository-local `.ce-ai.json` overrides with key-level precedence resolution over global `~/.config/ce-ai/state.json`.
- **Complete Multi-Harness Uninstall Parity (Issue #64)**: Extended `ce-ai uninstall` with `--harness <name|all>`, `--all`, and `--yes` / `-y` flags for complete removal of managed loaders and skills across all installed harnesses cleanly.
- **Teacher-Style Documentation**: Updated masterclass user guide with local cockpit settings vs master flight plan analogies.

---

## [0.6.0] - 2026-08-21

### Added
- **TUI Workflow Dashboard**: Interactive `🎮 Workflow (FSM)` panel in `ce-ai tui` visualizing 7-stage Flywheel status, active subtasks, and progress checkpoints.
- **Extended Companion Health Diagnostics**: Empirical health probes in `ce-ai doctor` for Engram SQLite DB (`~/.engram/engram.db`), CodeGraph index (`.codegraph/`), and RTK binary PATH.
- **Real-Time Sync Watcher**: Added `ce-ai sync --watch` flag for continuous drift monitoring and automatic SHA256 re-syncing.
- **Teacher-Style Documentation**: Updated masterclass user guides explaining cockpit instrument panels and autopilot guardrails.

---

## [0.5.0] - 2026-08-21

### Added
- **Workspace Scope Installation (Issue #7)**: Added `--scope workspace|global` flag in `ce-ai install` allowing repository-scoped skill installations (`./.opencode/`, `./.claude/`, `./.cursorrules`) resolved via `git rev-parse --show-toplevel`.
- **Companion Tools Manager (Issue #9)**: Added `ce-ai tools status` and `ce-ai tools install` for managing developer sidecars and memory servers (`Engram`, `CodeGraph`, `Context7`, `RTK`).
- **Workflow FSM & Recovery Engine (Issue #10)**: Added `ce-ai workflow status`, `ce-ai workflow checkpoint`, and `ce-ai workflow resume` tracking 7-stage development cycle progress and context recovery.
- **Automated GitHub Release Workflow**: Added `.github/workflows/release.yml` for multi-platform cross-compilation on `main` branch pushes.
- **Sync Verification Matrix**: Itemized SHA256 integrity reporting across all active host harnesses in `ce-ai sync`.

---

## [0.4.0] - 2026-08-21

### Added
- Interactive harness selection in TUI dashboard (`< [ target ] >`) supporting navigation across all 12 harness targets (`all`, `opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
- Dynamic host harness directory detection supporting `~/.claude`, `~/.pi`, `~/.kimi-code`, `~/.gemini/antigravity-cli`, and `~/.codex`.
- Interactive release version information display in TUI `Upgrade Release` tab.

### Fixed
- Fixed `state.json` recording bug in `src/commands/install.rs` where target harness names were previously hardcoded as `"opencode"`.
- Resolved Dependabot vulnerability alert #1 by upgrading `ratatui` to `0.30` (`lru` upgraded to `v0.18.2`).
- Resolved CodeQL workflow security alerts #1-#3 by adding top-level `permissions` block to `.github/workflows/ci.yml`.

### Governance
- Added mandatory Pull Request workflow directive to `AGENTS.md` prohibiting direct pushes to `main`.

### Added
- Multi-harness support (`HarnessKind` enum and `HarnessAdapter` trait) across 12 AI coding harness targets (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
- Native adapters in `src/harness/` for OpenCode, Claude Code, Pi, Cursor, Copilot, Generic JSON, and Custom fallback modes.
- Multi-harness model assignment sync (`ce-ai models set`) and `--all` host harness auto-probing.
- Expanded containerized Docker E2E test gate (`make e2e`).

### Added
- Pre-commit security gate (`.githooks/pre-commit` & `make hooks`) for secret scanning, test suites, and formatting checks.
- Automated PR rejection workflow (`auto-reject-failed-pr`) in GitHub Actions CI.
- Formal OpenSpec 7-stage development cycle and mandatory Stage 2 enforcement in `AGENTS.md`.
- Issue templates for security reports, feature requests, and harness support.
- OpenSpec roadmap items tracking GitHub Issues #1 through #10.

---

## [0.1.0] - 2026-08-20

### Added
- Core `ce-ai` CLI with `install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, and `doctor` subcommands.
- OpenCode harness integration with managed directory isolation and atomic write guarantees (`write_atomic`).
- Model profile snapshotting (`ce-ai models profile save/load`).
- SHA256 file manifest integrity tracking (`manifest.json`).
- Containerized Docker E2E gate suite (`make e2e`).
- Cross-platform CI GitHub Actions matrix (Linux, macOS, Windows).
