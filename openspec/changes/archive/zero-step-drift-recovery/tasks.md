# Tasks: Zero-Step Environment Drift Recovery via Live `RepoState` Sync

Work units carry per-unit changed-line estimates (~200 LOC target) so the PR-level forecast is derivable (CONTRIBUTING.md §4). Total forecast: ~220 lines.

- [x] **Task 1: Test Debt Coverage & Fixtures for `OpenSpecContextInfo` & `TreeDrift`** (~45 LOC)
  - [x] Add unit tests for `probe_openspec_context` in `src/commands/tests/workflow.rs` (testing explicit feature detection, task parsing, fallback to latest mtime, and missing dir resilience).
  - [x] Add unit tests for `TreeDrift` calculation in `src/commands/tests/sync.rs` (testing diff generation and summary formatting).
  - [x] Verification: `cargo test commands::tests::workflow && cargo test commands::tests::sync`

- [x] **Task 2: Data Model & Fast Probing Engine for `RepoState`** (~65 LOC)
  - [x] Define `RepoState` struct in `src/commands/workflow.rs` with `git_branch`, `head_sha`, `is_git_clean`, `modified_files`, `manifest_drift_count`, `adoption_status`, and `openspec_context`.
  - [x] Implement `probe_repo_state()` with fast git inspection (`git rev-parse`, `git status --porcelain=v1`), manifest diffing (`diff::diff`), and `AGENTS.md` block verification via SSOT.
  - [x] Implement fallback handling for non-git environments and missing manifest directories.
  - [x] Verification: `cargo test commands::tests::workflow`

- [x] **Task 3: Integration into `workflow resume` & `status`** (~50 LOC)
  - [x] Update `resume_lines()` in `src/commands/workflow.rs` to render `== [Environment State & Drift Status] ==` with branch, dirty working tree, and manifest drift warnings.
  - [x] Update `Action::Resume { json }` to serialize `repo_state` alongside `workflow` and `openspec_context`.
  - [x] Update `src/commands/status.rs` to surface git branch and dirty status in general status output.
  - [x] Verification: `cargo test --test cli`

- [x] **Task 4: Integration & Regression Tests for Zero-Step Drift Recovery** (~50 LOC)
  - [x] Add CLI integration tests in `tests/cli.rs`:
    - Clean repo turn resumption.
    - Drifted repo (modified files, switched branch) verified in Turn 0 without lag.
    - Drifted manifest warning output.
  - [x] Verification: `cargo test --test cli`

- [x] **Task 5: Version Bump, Changelog & Full Quality Gates** (~10 LOC)
  - [x] Bump version in `Cargo.toml` (`1.29.2` -> `1.30.0`).
  - [x] Update `CHANGELOG.md` following Keep a Changelog.
  - [x] Full quality gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.
