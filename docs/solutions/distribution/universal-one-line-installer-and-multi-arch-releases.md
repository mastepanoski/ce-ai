---
title: "Universal One-Line Installer Script & Multi-Arch Release Matrix"
category: "distribution"
date: "2026-08-21"
tags:
  - installer
  - multi-arch
  - release-pipeline
  - distribution
components:
  - scripts
  - workflows
applies_when: "Adding or maintaining universal installation scripts and multi-platform compilation workflows in ce-ai"
---

# Universal One-Line Installer Script & Multi-Arch Release Matrix

## Context

Prior to Release v0.8.0, installing `ce-ai` required building from source via `cargo install --path .`. To make `ce-ai` accessible on any system without requiring local Rust toolchains, `ce-ai` introduced a cross-platform release pipeline and universal installer script (`scripts/install.sh` and `scripts/install.ps1`).

---

## Guidance & Architecture Patterns

### 1. Multi-Arch Release Matrix (`.github/workflows/release.yml`)
- **Matrix Targets**:
  - `x86_64-unknown-linux-gnu` (Linux x86_64)
  - `aarch64-unknown-linux-gnu` (Linux ARM64)
  - `x86_64-apple-darwin` (macOS Intel)
  - `aarch64-apple-darwin` (macOS Apple Silicon)
  - `x86_64-pc-windows-msvc` (Windows x86_64)
  - `aarch64-pc-windows-msvc` (Windows ARM64)

### 2. POSIX-Compliant Universal Installer (`scripts/install.sh`)
- Auto-detects `uname -s` and `uname -m`.
- Fetches asset from `https://github.com/mastepanoski/ce-ai/releases/latest/download/ce-ai-${TARGET}.tar.gz`.
- Extracts binary directly into `~/.ce-ai/bin/ce-ai`.

---

## Usage Commands

### macOS & Linux One-Liner:
```bash
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash
```

### Windows PowerShell One-Liner:
```powershell
irm https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.ps1 | iex
```
