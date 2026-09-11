# Tasks: Workflow Feature Clearing on Reset

- [x] 1. Author failing unit tests for reset clearing, inheritance, and empty string override in `src/state/tests/state.rs` (~45 LOC)
- [x] 2. Implement feature resolution in `validate_and_set_workflow` (`src/state/state.rs`) (~25 LOC)
- [x] 3. Add defense-in-depth non-empty filtering in `probe_openspec_context_in` and correct `--help` doc comment in `src/commands/workflow.rs` (~15 LOC)
- [x] 4. Author CLI integration test reproducing full reset-to-stage-1 flow in `tests/cli.rs` (~40 LOC)
- [x] 5. Fix invalid copy-pasteable checkpoint commands in `docs/user-guide/quick-start-workflow-guide.md` and `docs/user-guide/fsm-and-checkpoints-explained.md` (~20 LOC)
- [x] 6. Bump SemVer to `1.37.1` in `Cargo.toml`, update `CHANGELOG.md`, and verify quality gates (~25 LOC)

Total estimated LOC: ~170 lines.
