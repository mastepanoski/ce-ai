# Proposal: Static Musl Linux Releases for Cross-Distro Compatibility

## Problem
The release pipeline builds Linux binaries on `ubuntu-latest` (Ubuntu 24.04, glibc 2.39) as
dynamically-linked `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` assets. A binary
produced this way hardcodes the build runner's glibc version. On any distro with an older
glibc the install fails at runtime:

```
ce-ai: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by ce-ai)
```

Concrete affected host: Debian 12 (glibc 2.36). Older glibc distros (Ubuntu 20/22, RHEL/CentOS 7)
are affected equally and progressively.

## In-Scope
- Change the two Linux release targets to fully static `*-unknown-linux-musl` assets.
- Build these with `cross` (musl container toolchain) so the binary is statically linked and runs
  on any Linux regardless of host glibc.
- Update `scripts/install.sh` to fetch the Linux `-musl` asset.
- Update `scripts/release-integrity.sh` SHA256 manifest asset names.
- Sync the distribution solution doc.
- Bump PATCH version + CHANGELOG entry (rule: merged fix MUST bump SemVer).

## Out-of-Scope
- macOS and Windows targets (not glibc-affected).
- Changing the install location / PATH behavior.
- Multi-version installer fallbacks (installer targets `latest`).

## Risk
- A `ring` (via `rustls-tls`) C dependency must compile under the musl toolchain. `cross`'s musl
  images ship the required `*-linux-musl-gcc`; this is a well-proven path.
- No rollback of already-published `-gnu` assets: they remain on past tags. Only `latest` and new
  tags gain `-musl` assets. Old `-gnu` assets are never removed, so no release is broken.

## Rollback
Revert commit restores the `-gnu` target lines, the `cross` steps, and the `-gnu` asset names in
`install.sh`/`release-integrity.sh`; existing `-gnu` release assets remain available on prior tags.

## Success Criteria
- `scripts/install.sh` on a glibc 2.36 host (Debian 12) installs a `ce-ai` binary with no
  `GLIBC_2.39 not found` error.
- `readelf -s ce-ai | grep GLIBC` shows only `GLIBC_2.2.5`-era baseline symbols (or musl, no gnu
  glibc requirement) for Linux artifacts.
- `release-integrity.sh` emits a SHA256SUMS.txt covering both `-musl` Linux assets.
