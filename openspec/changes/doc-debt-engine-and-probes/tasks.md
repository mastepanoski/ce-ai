# Tasks: Documentation Technical Debt Diagnostic Engine & Probes (Increments 1 & 2)

Work-unit changed-line estimates total: ~940 LOC (~200 LOC/work-unit policy applied to code-bearing units; tests/docs are additive verification).

- [x] **Work Unit 1: Substrate Tri-State & Umbrella Models in `src/commands/workflow.rs`** (~160 LOC)
  - [x] 1.1 Define `ProbeStatus<T>` enum with `Clean`, `Debt(T)`, and `Unknown` variants and helper methods.
  - [x] 1.2 Define finding models: `OpenSpecDesyncFinding`, `DesyncReason`, `StalePendingFinding`, `InactivitySource`, `SolutionDriftFinding`.
  - [x] 1.3 Define `DocDebtReport` struct aggregating the three probes and `git_available` boolean.
  - [x] 1.4 Add `pub doc_debt: Option<DocDebtReport>` to `RepoState`.
  - [x] 1.5 Implement serde serialization/deserialization for all new types.
  - [x] 1.6 (TDD) In `src/commands/tests/workflow.rs`, add unit tests validating tri-state serialization round-trip and `Unknown` handling when Git is absent.

- [x] **Work Unit 2: Workspace Config Overrides in `src/state/state.rs` & `src/state/ports.rs`** (~110 LOC)
  - [x] 2.1 Define `DocHygieneConfig` struct with `stale_spec_days`, `check_solution_paths`, and `require_solution_frontmatter`.
  - [x] 2.2 Add `pub doc_hygiene: Option<DocHygieneConfig>` to `State`.
  - [x] 2.3 Update `State::merge_overrides` to merge `doc_hygiene` from workspace `.ce-ai.json`.
  - [x] 2.4 (TDD) In `src/state/tests/state.rs`, add unit tests verifying `.ce-ai.json` overrides merge cleanly and default values apply when omitted.

- [x] **Work Unit 3: Probe 1 (Git-Aware Desync) & Probe 2 (Stale Inactivity Watchdog)** (~190 LOC)
  - [x] 3.1 Implement `probe_openspec_desync(repo_root: &Path, git_available: bool) -> ProbeStatus<Vec<OpenSpecDesyncFinding>>`.
    - Check parent tasks complete with subtasks open.
    - Inspect git commits landed on HEAD/main.
    - Check if cited version in tasks.md is surpassed in Cargo.toml.
  - [x] 3.2 Implement `probe_stale_pending_openspecs(repo_root: &Path, stale_days: u32, git_available: bool) -> ProbeStatus<Vec<StalePendingFinding>>`.
    - Compute elapsed days via git commit timestamp, falling back to mtime in No-Git environments.
  - [x] 3.3 (TDD) In `src/commands/tests/workflow.rs`, add unit tests covering:
    - Stranded changes fixture (`doctor-kimi-marketplace-divergence` shape).
    - Stale pending change fixture with simulated mtime/git commit age.
    - No-Git fixture returning `Unknown` for git-dependent checks.

- [x] **Work Unit 4: Probe 5 (Solution Library Drift & YAML Linter)** (~150 LOC)
  - [x] 4.1 Implement `probe_solution_drift(repo_root: &Path, config: &DocHygieneConfig) -> ProbeStatus<Vec<SolutionDriftFinding>>`.
    - Extract backticked paths (`src/**/*.rs`, `tests/**/*.rs`) and verify existence.
    - Validate YAML frontmatter keys (`title`, `category`/`module`, `problem_type`, `tags`, `applies_when`).
  - [x] 4.2 (TDD) In `src/commands/tests/workflow.rs`, add unit tests covering:
    - Dead file path in solution detection.
    - Missing YAML frontmatter fields detection.
    - Clean solutions directory returning `ProbeStatus::Clean`.

- [ ] **Work Unit 5: Turn-0 Delivery & Doctor Integration** (~180 LOC)
  - [ ] 5.1 Implement `probe_doc_debt` coordinating all three probes and wire into `probe_repo_state`.
  - [ ] 5.2 Format single-line Turn-0 summary in `workflow resume` and `workflow status` with `[git: n/a]` fallback.
  - [ ] 5.3 In `src/commands/doctor.rs`, emit non-blocking `doctor-warn:` lines with copy-pasteable CLI commands for detected debt items.
  - [ ] 5.4 (TDD) Add integration tests in `src/commands/tests/doctor.rs` and `tests/cli.rs` validating doctor output with and without Git substrate.

- [ ] **Work Unit 6: Pedagogical Guide & Documentation Updates** (~150 LOC)
  - [ ] 6.1 Author `docs/user-guide/doc-hygiene-and-debt-explained.md` in Diátaxis Explanation quadrant for Beginners ("Professor's Walkthrough").
  - [ ] 6.2 Update `README.md` introducing documentation debt and linking to the new guide in the Documentation Map.
  - [ ] 6.3 Verify `README.md <= 100` lines constraint (`wc -l README.md`).
  - [ ] 6.4 Update `CONCEPTS.md` with definitions for `Documentation Technical Debt Diagnostic Engine`, `Substrate Tri-State`, and `Solution Drift Probe`.
  - [ ] 6.5 Update `CHANGELOG.md` under `[Unreleased]`.
  - [ ] 6.6 Run full verification gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
