# Tasks: Antigravity PreInvocation Turn-0 Drift Delivery

- [x] 1. Add `--pre-invocation` flag and handler with session deduplication in `src/commands/workflow.rs` (~50 LOC)
- [x] 2. Implement `has_pre_invocation_hook`, `ensure_pre_invocation_hook`, `remove_pre_invocation_hook` in `src/harness/agy.rs` (~110 LOC)
- [x] 3. Implement unit tests in `src/harness/tests/agy.rs` (~75 LOC)
- [x] 4. Wire hook in `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, and `src/commands/doctor.rs` (~40 LOC)
- [x] 5. Implement CLI integration test in `tests/cli.rs` (~65 LOC)
- [x] 6. Update user documentation in `docs/user-guide/zero-step-drift-recovery-explained.md` and capture solution architecture in `docs/solutions/architecture/` (~50 LOC)
- [x] 7. Bump SemVer to `1.37.0` in `Cargo.toml` and update `CHANGELOG.md` (~20 LOC)

Total estimated LOC: ~410 lines (use `size:exception` label if needed for tests/docs).
