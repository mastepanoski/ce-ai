# Exploration: Cargo Build Cache and Artifact Optimization

## Technical Context
In Rust/Cargo projects with large integration test suites (such as `ce-ai`'s `tests/cli.rs` at 10,000+ LOC, `tests/security.rs`, `tests/e2e.rs`), compiling with `dev` profile defaults leads to severe artifact accumulation:

1. **Incremental Compilation Overhead**:
   Cargo creates a session directory under `target/debug/incremental/` per target (binary, lib, each integration test). Every time source code changes, a new hash directory is created. Over 700 such directories accumulated, consuming 27 GB.
   Setting `incremental = false` disables the incremental cache graph, slightly increasing clean rebuild time from scratch but entirely halting the 27 GB disk consumption.

2. **DWARF Debug Info Level (`debug` setting)**:
   - `debug = 2` (default for `dev` and `test`): Generates full DWARF symbols including type tables, variable locations, and AST references.
   - `debug = "line-tables-only"` (level 1): Generates source line mappings without variable and type metadata.
   - For automated test suites and agent debugging, line numbers and file names are all that is required for panics and `RUST_BACKTRACE=1`.
   - Stripping full type/variable metadata reduces `.rlib` and test binary size by 70–85%.

3. **Pruning Tooling (`cargo-sweep`)**:
   `cargo-sweep` is the standard tool in the Rust ecosystem to delete build artifacts older than a given threshold (e.g. `--time 14` days) or belonging to old toolchains, without requiring a complete rebuild.
