# Tasks: Optimize Cargo Build Cache and Artifact Storage

- [x] **Unit 1 (~20 LOC):** Configure compact dev/test profiles in `Cargo.toml`
  - Add `[profile.dev]` with `debug = "line-tables-only"` and `incremental = false`.
  - Add `[profile.test]` with `debug = "line-tables-only"` and `incremental = false`.
  - *Verification*: `cargo check` and `cargo test --lib` build without warnings and produce compact artifacts.

- [x] **Unit 2 (~25 LOC):** Add `sweep` target in `Makefile` & bump patch version
  - Add `sweep` recipe checking/installing `cargo-sweep` and running `cargo sweep --time 14`.
  - Bump SemVer to `1.73.1` in `Cargo.toml`.
  - Update `CHANGELOG.md` with build optimization notes.
  - *Verification*: `make sweep --dry-run` or help verification, `cargo fmt --check`, `cargo clippy`, `cargo test`, `make e2e`, and `ce-ai doc lint --strict`.
