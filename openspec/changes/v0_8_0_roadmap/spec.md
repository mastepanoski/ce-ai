# OpenSpec Requirements: Release v0.8.0 Specifications

## Feature 1: Multi-Platform Native Binary Compilation (Issue #28)

### Requirement 1.1: Multi-Target CI Matrix
- **WHEN** a release tag matching `v*` is pushed to GitHub,
- **THEN** GitHub Actions MUST compile release binaries for 6 target architectures: `x86_64-linux`, `aarch64-linux`, `x86_64-macos`, `aarch64-macos` (Apple Silicon), `x86_64-windows`, and `aarch64-windows`.

### Requirement 1.2: Checksum Manifest Generation
- **WHEN** binary compilation completes for each target,
- **THEN** CI MUST generate SHA256 checksum files and publish tarball assets to the GitHub Release.

---

## Feature 2: Universal One-Line Installer Script (Issue #3)

### Requirement 2.1: Automatic OS & Architecture Detection
- **WHEN** `scripts/install.sh` is executed on macOS or Linux,
- **THEN** the script MUST detect `uname -s` and `uname -m` and select the matching binary release asset URL.

### Requirement 2.2: Zero-Dependency Extraction & PATH Installation
- **WHEN** the tarball is fetched,
- **THEN** `install.sh` MUST extract the binary to `~/.ce-ai/bin/ce-ai`, set executable permissions, and output PATH instructions.

---

## Feature 3: Homebrew Formula Specification (Issue #2)

### Requirement 3.1: Homebrew Formula Template
- **WHEN** `Formula/ce-ai.rb` is requested,
- **THEN** a Homebrew formula file MUST be available in the repository defining binary url, sha256, and installation steps.
