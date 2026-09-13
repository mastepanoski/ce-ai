# Specification: Static Musl Linux Releases

## Requirements Matrix

### REQ-1: Static Linux Release Targets
- **WHEN** the release workflow runs for a Linux x86_64 host
- **THEN** it SHALL build `x86_64-unknown-linux-musl` and SHOULD produce asset `ce-ai-x86_64-unknown-linux-musl.tar.gz`.
- **WHEN** the release workflow runs for a Linux aarch64 (ARM64) host
- **THEN** it SHALL build `aarch64-unknown-linux-musl` and SHOULD produce asset `ce-ai-aarch64-unknown-linux-musl.tar.gz`.

### REQ-2: Cross-Based Static Build
- **WHEN** the release workflow builds a Linux musl target
- **THEN** it SHALL use `cross build --release --target <target>` (via `taiki-e/install-action@cross`), which produces a statically-linked binary with no host glibc requirement.
- **WHEN** the release workflow builds a non-Linux target (macOS/Windows)
- **THEN** it SHALL keep using `cargo build --release --target <target>`, unchanged.

### REQ-3: Installer Fetches Musl Asset on Linux
- **WHEN** `scripts/install.sh` detects the Linux OS
- **THEN** it SHALL set `TARGET` to `${ARCH_NAME}-unknown-linux-musl` and `ASSET_NAME` to `ce-ai-${TARGET}.tar.gz`.

### REQ-4: Integrity Manifest Covers Musl Assets
- **WHEN** `scripts/release-integrity.sh` computes SHA256SUMS for a release
- **THEN** the `ASSETS` array SHALL include `ce-ai-x86_64-unknown-linux-musl.tar.gz` and `ce-ai-aarch64-unknown-linux-musl.tar.gz` (and no longer the `-gnu` variants).

### REQ-5: No glibc>baseline Runtime Requirement
- **WHEN** the Linux `-musl` binary is inspected
- **THEN** it SHALL NOT require a glibc symbol newer than a portable baseline for the distro it targets (musl statically links libc).

### REQ-6: Preservation of Existing Releases
- **WHEN** a new release is published
- **THEN** previously published `-gnu` assets SHALL remain available on their original tags (no deletion, no breakage).
