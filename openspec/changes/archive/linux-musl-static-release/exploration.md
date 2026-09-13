# Exploration: Static Musl Linux Releases

## Investigation
- `ldd --version` on the affected host: Debian GLIBC 2.36 (Debian 12 bookworm).
- `readelf -s ~/.ce-ai/bin/ce-ai | grep GLIBC` showed a required `GLIBC_2.39` symbol.
- `release.yml` publishes `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu` on
  `ubuntu-latest` (Ubuntu 24.04 → glibc 2.39). Root cause: dynamically-linked glibc binary tied to
  the build runner's glibc.

## Options Considered

### A. Static musl assets (chosen)
- Build `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` via `cross`, producing a fully
  static binary with no glibc dependency at runtime.
- Pros: runs on ANY Linux regardless of glibc. Solves the class of bug permanently.
- Cons: `ring` (via `rustls-tls`) requires a musl C compiler; cross's musl container supplies
  `x86_64-linux-musl-gcc` / `aarch64-linux-musl-gcc`. Asset/provisioning names change.

### B. Pin runner to `ubuntu-22.04`
- Minimal change, but glibc 2.35 baseline still breaks Ubuntu 20.04 and RHEL/CentOS 7. Only a
  deferral, not a fix of the class.

### C. Zigbuild with low glibc baseline (e.g. 2.28)
- Keeps `-gnu` asset name and broad compat, but adds a zig toolchain dependency and hand-tuned
  baseline pins; more CI complexity for marginal gain over fully-static musl.

## Tradeoff Decision
Musl static (A) is chosen: it is the only option that fully removes the host-glibc coupling and
requires no version-specific baseline maintenance. The `ring`/musl link path is industry-standard
and verified by a `cargo check --target x86_64-unknown-linux-musl` dep-graph compile (link needs the
musl cross C toolchain supplied by `cross` in CI).
