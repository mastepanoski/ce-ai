# Tasks: Cursor sessionStart Lifecycle Hook Integration

- [ ] 1. Enhance `src/commands/workflow.rs` resume JSON payload with `additional_context` (~15 LOC)
- [ ] 2. Implement `has_session_start_hook`, `ensure_session_start_hook`, `remove_session_start_hook` in `src/harness/cursor.rs` (~110 LOC)
- [ ] 3. Implement unit tests in `src/harness/tests/cursor.rs` (~70 LOC)
- [ ] 4. Wire hook in `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, and `src/commands/doctor.rs` (~45 LOC)
- [ ] 5. Implement CLI integration test in `tests/cli.rs` (~60 LOC)
- [ ] 6. Update user documentation in `docs/user-guide/zero-step-drift-recovery-explained.md` and capture knowledge in `docs/solutions/architecture/` (~70 LOC)
- [ ] 7. Bump SemVer to `1.36.0` in `Cargo.toml` and update `CHANGELOG.md` (~20 LOC)

Total estimated LOC: ~390 lines (within the ~400 LOC budget).
