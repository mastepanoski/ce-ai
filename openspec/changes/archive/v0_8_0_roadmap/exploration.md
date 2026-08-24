# OpenSpec Exploration: Release v0.8.0 Technical Investigation

## Technical Alternatives Evaluated

### Option A: `cargo-dist` Automation
- *Approach*: Use `cargo-dist` CLI for generating GitHub Actions workflows and release artifacts.
- *Tradeoff*: Requires external tool installation and strict release configuration schema.
- *Decision*: Adopt `cargo-dist` / native GitHub Actions matrix compilation with `cross` for full control over ARM64 and Windows targets.

### Option B: Universal Installer Script (`install.sh`)
- *Approach*: Write a POSIX-compliant shell script that queries GitHub Release API (`https://api.github.com/repos/mastepanoski/ce-ai/releases/latest`), extracts OS/arch, downloads the tar.gz tarball, verifies SHA256, and installs to `~/.ce-ai/bin/`.
- *Rationale*: Zero-dependency installation path working out-of-the-box on macOS and Linux without requiring Rust/Cargo.

---

## Architectural Tradeoffs & Conclusions

- **Matrix Targets**:
  - `x86_64-unknown-linux-gnu` (Linux x86_64)
  - `aarch64-unknown-linux-gnu` (Linux ARM64)
  - `x86_64-apple-darwin` (macOS Intel)
  - `aarch64-apple-darwin` (macOS Apple Silicon)
  - `x86_64-pc-windows-msvc` (Windows x86_64)
  - `aarch64-pc-windows-msvc` (Windows ARM64)
- **Checksum Verification**:
  - SHA256 checksums written to `SHA256SUMS.txt` and verified prior to extraction.
