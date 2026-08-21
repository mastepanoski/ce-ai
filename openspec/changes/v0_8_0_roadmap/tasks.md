# OpenSpec Tasks: Release v0.8.0 Implementation Plan

- [x] **Unit 1: GitHub Release Workflow Matrix (`.github/workflows/release.yml`)**
  - [x] Add 6-target build matrix (`x86_64-linux`, `aarch64-linux`, `x86_64-macos`, `aarch64-macos`, `x86_64-windows`, `aarch64-windows`).
  - [x] Generate SHA256 checksum manifest per tarball.
  - [x] Upload release assets via `softprops/action-gh-release@v2`.

- [x] **Unit 2: Universal One-Line Installer Script (`scripts/install.sh` & `scripts/install.ps1`)**
  - [x] Create POSIX shell installer script `scripts/install.sh` supporting macOS and Linux (x86_64 & ARM64).
  - [x] Create PowerShell installer script `scripts/install.ps1` for Windows.

- [x] **Unit 3: Package Manager Homebrew Formula (`Formula/ce-ai.rb`)**
  - [x] Create Homebrew formula template `Formula/ce-ai.rb`.

- [x] **Unit 4: Documentation & Release Preparation**
  - [x] Update `README.md` with one-line installation instructions (`curl -fsSL ... | bash`).
  - [x] Update `ROADMAP.md` setting `v0.8.0` status to In Progress.
