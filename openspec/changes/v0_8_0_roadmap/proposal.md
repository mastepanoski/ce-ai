# OpenSpec Proposal: Release v0.8.0 — Automated CI/CD Release Pipeline & Distribution

## Problem Statement
While `ce-ai` v0.7.0 delivered repository-level workspace overrides and complete multi-harness uninstallation parity, binary compilation and installation currently require local Rust toolchains (`cargo install`). To achieve seamless distribution for end users across all major operating systems and architectures, `ce-ai` requires an automated multi-platform release pipeline and distribution mechanism:

1. **Multi-Platform Native Binaries (Issue #28)**: Automated GitHub Actions CI workflow compiling and attaching signed native release binaries for:
   - Linux `x86_64-unknown-linux-gnu` & `aarch64-unknown-linux-gnu` (ARM64)
   - macOS `x86_64-apple-darwin` & `aarch64-apple-darwin` (Apple Silicon)
   - Windows `x86_64-pc-windows-msvc` & `aarch64-pc-windows-msvc` (ARM64)
2. **Universal One-Line Installer Script (Issue #3)**: A cross-platform shell/PowerShell installer script (`curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/install.sh | bash`) that auto-detects OS and architecture, downloads the latest compiled binary from GitHub Releases, and places it in the user's `PATH` (`~/.ce-ai/bin/`).
3. **Package Manager Formulas (Issue #2)**: Distribution specs and formula templates for Homebrew (`homebrew-ce-ai`), WinGet, and APT packages.

## In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **GitHub Release CI/CD Workflow (`.github/workflows/release.yml`)**:
  - Triggers automatically on tag push (`v*`).
  - Cross-compiles target binaries using `matrix` builds and `cross` / `cargo-zigbuild`.
  - Generates `SHA256SUMS.txt` checksum manifest and attaches all assets to the GitHub Release.
- **Universal Installer Script (`scripts/install.sh` & `scripts/install.ps1`)**:
  - Auto-detects OS (`darwin`, `linux`, `windows`) and architecture (`x86_64`, `aarch64`).
  - Downloads binary & verifies SHA256 checksum against `SHA256SUMS.txt`.
  - Installs to `~/.ce-ai/bin/` and updates `PATH` hints.
- **Package Manager Specification (Issue #2)**:
  - Homebrew tap formula spec (`Formula/ce-ai.rb`).

### Out-of-Scope:
- ISO 27001 penetration test audit (deferred to `v0.9.0`).
- Frozen 1.0 API contract freeze (deferred to `v1.0.0`).

## Success Criteria
1. Pushing tag `v0.8.0` triggers GitHub Actions CI to cross-compile 6 target binaries, generate SHA256 manifests, and publish release assets cleanly.
2. `curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/install.sh | bash` successfully installs `ce-ai` on macOS (Intel/Apple Silicon) and Linux (x86_64/ARM64).
3. Issues #2, #3, and #28 closed cleanly with 100% green CI.
