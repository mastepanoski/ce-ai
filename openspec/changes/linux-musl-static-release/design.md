# Design: Static Musl Linux Releases

## Architecture
The release matrix in `.github/workflows/release.yml` selects an `os` per target. The two Linux
entries move from glibc targets to musl targets and are built with `cross` instead of `cargo`.

### Release Matrix (Linux)
| target | os | asset_name |
|--------|----|-----------|
| `x86_64-unknown-linux-musl` | ubuntu-latest | `ce-ai-x86_64-unknown-linux-musl.tar.gz` |
| `aarch64-unknown-linux-musl` | ubuntu-latest | `ce-ai-aarch64-unknown-linux-musl.tar.gz` |

### Build Step Selection (sequence)
```
matrix.os == 'ubuntu-latest'
  └─ install cross (taiki-e/install-action@cross)
  └─ cross build --release --target <musl-target>   # static binary
matrix.os != 'ubuntu-latest'  (mac/windows)
  └─ cargo build --release --target <target>        # unchanged
```

`cross` runs the build inside its musl container image, which ships the target's musl C compiler so
`ring` links statically. Output remains at `target/<target>/release/ce-ai`, so the existing
`Package Binary (Unix)` tar step is unchanged.

### Installer (scripts/install.sh)
Linux `TARGET` becomes `${ARCH_NAME}-unknown-linux-musl`; `ASSET_NAME` derives the `-musl` tar.gz.
The API-resolved `browser_download_url` grep and retry/fallback logic are unchanged — they already
match whatever `ASSET_NAME` is.

### Integrity (scripts/release-integrity.sh)
The two Linux entries in the `ASSETS` array change to the `-musl` names so SHA256SUMS.txt covers the
actual published artifacts.

## Rationale
- Fully static binary → no dependency on host glibc version. Fixes the bug class, not one distro.
- `cross` abstracts the per-target C/musl toolchain, avoiding hand-maintained cross-compiler and
  linker env vars that the previous `gcc-aarch64-linux-gnu` ARM step needed.

## ADR
- ADR-1: Linux artifacts SHALL be static `*-unknown-linux-musl`, built via `cross`.
- ADR-2: macOS/Windows targets SHALL remain unchanged (native dynamic).
