# Tasks: Retiring Ratatui TUI & Dispatch Simplification

<!-- Work units carry per-unit changed-line estimates (~200 LOC target) per AGENTS.md / CONTRIBUTING.md §4 -->

- [x] **Task 1: Dependency Pruning (`Cargo.toml`)** [~10 LOC changed, -200 LOC in Cargo.lock]
  - [x] Remove `ratatui = "0.30"` from `Cargo.toml`.
  - [x] Remove `crossterm = "0.28"` from `Cargo.toml`.
  - [x] Update `Cargo.lock` via `cargo check` / `cargo update`.

- [x] **Task 2: Delete `src/tui/` Subsystem** [-1,835 LOC deleted]
  - [x] Remove `pub mod tui;` from `src/lib.rs`.
  - [x] Delete `src/tui/` directory and all 8 contained files.

- [x] **Task 3: Update CLI Entry Point & Dispatch** [~15 LOC changed]
  - [x] Update `src/commands/registry.rs:dispatch` so that `None => status::run(ctx)`.
  - [x] Update docstrings and comments referencing `tui`.

- [x] **Task 4: CLI Integration Tests** [~30 LOC added]
  - [x] Add integration test in `tests/cli.rs` validating that running `ce-ai` without arguments executes `status` cleanly and exits with code 0.
  - [x] Verify help text (`ce-ai --help`) works as expected.

- [x] **Task 5: Documentation & Changelog Hygiene** [~40 LOC changed]
  - [x] Remove `docs/user-guide/workflow-panel-native-vs-agent-skills.md`.
  - [x] Update `README.md` to remove the workflow panel entry.
  - [x] Bump version in `Cargo.toml` (`1.71.0` -> `1.72.0`) per SemVer minor for feature/refactoring.
  - [x] Document changes in `CHANGELOG.md`.

- [x] **Task 6: Verification & Quality Gate** [0 LOC]
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e` (if docker is available).
