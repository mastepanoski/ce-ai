# Tasks: Decommission ODD Subsystem and Align Compound Agile Fast-Path

- [x] **Work Unit 1: Workflow Banners and CLI Guidance Alignment** (~80 LOC)
  - [x] Update `status_lines_with_mode` in `src/commands/workflow.rs` to render the Compound Agile Fast-Path banner.
  - [x] Add deprecation warning notice in `handle_graduate` in `src/commands/workflow.rs`.
  - [x] Update `guide_lines` in `src/commands/workflow.rs` to describe the Compound Agile Fast-Path and prune `graduate`.
  - [x] Update advisory notice in `src/commands/gate.rs` line 656.

- [x] **Work Unit 2: Test Suite Alignment and Regression Verification** (~90 LOC)
  - [x] Update unit test `test_status_and_resume_banners_with_mode` in `src/commands/tests/workflow.rs`.
  - [x] Update integration tests in `tests/cli.rs` (`test_cli_odd_mode_router_and_graduation_lifecycle`).
  - [x] Run full test suite (`cargo test`) across all 863+ tests.

- [x] **Work Unit 3: Documentation and Architectural Learning Capture** (~120 LOC)
  - [x] Update `docs/user-guide/odd-fast-path-and-graduation-masterclass.md` to reflect the Compound Agile Fast-Path.
  - [x] Update `docs/user-guide/ce-ai-positioning.md` and `docs/user-guide/cli-reference.md`.
  - [x] Accrete *Compound Agile Fast-Path* in `CONCEPTS.md` and verify with `cargo run -- doc lint --strict`.
  - [x] Document solution in `docs/solutions/architecture/decommission-odd-and-align-compound-fast-path.md`.
  - [x] Bump version to `3.5.0` in `Cargo.toml` and update `CHANGELOG.md`.

- [x] **Unit Final: Lifecycle Completion & Governance Gate**
  - [x] **Simplification (`ce-simplify-code`)**: Audit changed code for reuse, quality, and efficiency while preserving behavior.
  - [x] **Revisión formal (`ce-code-review`)**: Run review and record `ce-ai workflow review-receipt`.
  - [x] **Captura de conocimiento (`ce-compound`)**: Capture learnings in `docs/solutions/<category>/` and monotonically accrete `CONCEPTS.md`.
  - [x] **Verificación de cierre**: Comply with DoD (`cargo fmt`, `cargo clippy`, `cargo test`, `make e2e`, `doc lint --strict`).
