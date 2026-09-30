# Proposal: Optimize Cargo Build Cache and Artifact Storage

## Problem Statement
The `target/` directory in `ce-ai` ballooned to over 67 GiB across 234,000 files due to:
1. `incremental/` storing cache directories for hundreds of iterative test runs (~27 GB).
2. `deps/` retaining dozens of unpruned versions of `libce_ai-<hash>.rlib` and test binaries (~15 GB).
3. Default `dev` and `test` profile compilation generating full DWARF debug info (`debug = 2`), bloating each compilation artifact with symbol, type, and local variable metadata.

## In-Scope
- Configure `[profile.dev]` and `[profile.test]` in `Cargo.toml` with `debug = "line-tables-only"` (preserving file and line numbers for stack traces while stripping variable/type metadata).
- Configure `incremental = false` in `[profile.dev]` and `[profile.test]` to eliminate `target/debug/incremental/` bloat across repeated test iterations.
- Add `sweep` target in `Makefile` to invoke `cargo sweep --time 14`.
- Document disk optimization best practices in `CHANGELOG.md` and docs.

## Out-of-Scope
- Modifying release profile optimization levels or stripping release binaries.
- Changing CI compiler flags outside standard profile inheritance.

## Success Criteria
- `target/debug` size stays small without unboundedly growing across test runs.
- Backtraces (`RUST_BACKTRACE=1`) still show exact file paths and line numbers.
- 100% test pass rate across all local and CI matrix jobs.
