# Tasks: Static Musl Linux Releases

## Phase 1 — Build Pipeline
- [x] 1.1 `.github/workflows/release.yml`: change Linux matrix entries to `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` with `ce-ai-*-unknown-linux-musl.tar.gz` asset names.
- [x] 1.2 `.github/workflows/release.yml`: replace the ARM64 GCC cross-compiler step and the single build step with `cross` install + `cross build` for `ubuntu-latest`, and keep `cargo build` for non-Linux.
- [x] 1.3 `.github/workflows/release.yml`: verify packaging step still reads `target/<target>/release/ce-ai` (unchanged).

## Phase 2 — Installer & Integrity
- [x] 2.1 `scripts/install.sh`: Linux `TARGET` → `${ARCH_NAME}-unknown-linux-musl`.
- [x] 2.2 `scripts/release-integrity.sh`: `ASSETS` Linux entries → `-musl` names.
- [x] 2.3 `docs/solutions/distribution/universal-one-line-installer-and-multi-arch-releases.md`: update matrix targets and add the musl rationale.

## Phase 3 — Versioning & Verification
- [x] 3.1 `Cargo.toml`: bump `version` to the next PATCH (`1.50.1`).
- [x] 3.2 `CHANGELOG.md`: add an `[1.50.1]` entry under "Fixed" describing the static musl Linux release change.
- [x] 3.3 `cargo check --target x86_64-unknown-linux-musl` (dep graph compiles; link requires musl toolchain supplied by `cross` in CI).
- [x] 3.4 `cargo fmt --check` passes.
- [x] 3.5 `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] 3.6 `cargo test` passes.
- [x] 3.7 Release tag v1.50.1 builds both `-musl` assets and `release-integrity.sh` emits SHA256SUMS.txt covering them (verified in CI on a real tag).
