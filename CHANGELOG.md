# Changelog

All notable changes to `ce-ai` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

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
