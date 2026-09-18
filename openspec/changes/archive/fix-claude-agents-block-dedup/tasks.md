# Tasks: Skip Managed Block Injection into CLAUDE.md When Delegating to AGENTS.md

- [x] **Work Unit 1: Delegation Parser and Harness Reconciliation** (~120 LOC)
  - [x] Implement `delegates_to_agents_md(content: &str) -> bool` in `src/harness/claude.rs`.
  - [x] Update `reconcile_project_harness_hooks` in `src/commands/init_prj.rs` to skip `update_claude_md` when delegation to `AGENTS.md` is active and strip existing duplicates.
  - [x] Update `deinit_prj.rs` to clean up `@AGENTS.md` stub files on project de-adoption.
  - [x] Verification: `cargo check`.

- [x] **Work Unit 2: Unit & Integration Test Suite** (~150 LOC)
  - [x] Add unit tests for `delegates_to_agents_md` in `src/harness/tests/claude.rs`.
  - [x] Add unit tests for `reconcile_project_harness_hooks` in `src/commands/tests/init_prj.rs` for:
    - No injection when `@AGENTS.md` stub is present.
    - Stripping existing duplicate `CE_MANAGED_BEGIN` block while keeping `@AGENTS.md` and user directives.
    - Normal injection when `@AGENTS.md` is absent.
  - [x] Add integration test in `tests/cli.rs` verifying `init-prj` and `sync` on a project with `.claude/` present.
  - [x] Verification: `cargo test`.

- [x] **Work Unit 3: DoD Quality Gates, Versioning & CHANGELOG** (~30 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e`.
  - [x] Bump patch version in `Cargo.toml`.
  - [x] Document the fix in `CHANGELOG.md` referencing issue #377.
