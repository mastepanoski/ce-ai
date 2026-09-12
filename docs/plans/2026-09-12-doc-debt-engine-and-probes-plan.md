---
date: 2026-09-12
topic: doc-debt-engine-and-probes
status: ready
source: docs/brainstorms/2026-09-12-doc-debt-engine-and-probes-requirements.md
---

# Plan: Documentation Technical Debt Diagnostic Engine & Probes (Increments 1 & 2)

## Problem Frame & Scope

In Compound Engineering, documentation acts as the project's living contract and memory layer. As code evolves rapidly across releases, documentation inevitably accumulates technical debt:
1. **Subtask Desync Stranding Completed Features:** `probe_unarchived_completed_changes` in `src/commands/workflow.rs` strictly checks `completed == total`. Features with 100% top-level tasks checked (`- [x] Task 1`) but open subtasks (`- [ ] 1.1`), or with code already merged to `main` while a post-merge release checkbox remains open (e.g. `doctor-kimi-marketplace-divergence`, `harness-manifest-sha256-coverage`, and `linux-musl-static-release`), are ignored by `doctor` and linger indefinitely in `openspec/changes/`.
2. **Silent OpenSpec Inactivity Rot:** Changes abandoned or stalled with 0% or partial task completion receive zero diagnostics from `doctor`, cluttering active change directories.
3. **Solution Library Drift in `docs/solutions/`:** Refactorings and file moves render past solution file paths obsolete. AI coding agents searching `docs/solutions/` ingest dead file pointers and hallucinate deleted APIs or superseded patterns.
4. **Substrate Concealment Antipattern:** Collapsing missing substrate (such as environments without `.git`) to `clean` or `0` creates false confidence by equating absence of diagnostic signal with absence of debt.
5. **Newbie Comprehension Gap:** Without an accessible, pedagogical explanation of *why* doc rot misleads AI agents and *how* automated probes protect development velocity, beginners struggle to understand the value of doc hygiene.

This plan establishes a unified, non-blocking documentation technical debt diagnostic engine across `RepoState`, `workflow resume`, `workflow status`, and `ce-ai doctor`, with first-class No-Git resilience (tri-state reporting), three core probes, a pedagogical newbie guide, and an updated `README.md` (≤ 100 lines). Active compaction (`ce-ai archive compact`) is explicitly decoupled into a separate pull request.

## Requirements Traceability

- **R1 (Substrate Tri-State `ProbeStatus<T>`):** Covered in Unit 1.
- **R2 (Umbrella `DocDebtReport` in `RepoState`):** Covered in Unit 1.
- **R3 (Workspace Override Thresholds in `.ce-ai.json`):** Covered in Unit 2.
- **R4 (Probe 1 — Git-Aware OpenSpec Desync & Landing):** Covered in Unit 3.
- **R5 (Probe 2 — Stale Pending OpenSpec Decay Watchdog):** Covered in Unit 3.
- **R6 (Probe 5 — Solution Library Drift Probe):** Covered in Unit 4.
- **R7 (Actionable Diagnostic Delivery in `doctor`):** Covered in Unit 5.
- **R8 (Compact Turn-0 Banner in `workflow resume` & `status`):** Covered in Unit 5.
- **R9 (Pedagogical "Professor-Style" Guide for Newbies):** Covered in Unit 6.
- **R10 (Newbie-Friendly `README.md` & Documentation Map Update):** Covered in Unit 6.

## Implementation Units

### Unit 1: Substrate Tri-State & Umbrella Models in `src/commands/workflow.rs`
- **Files:** `src/commands/workflow.rs`, `src/commands/tests/workflow.rs`
- **Target:** ~160 LOC
- **Approach:**
  - Define generic enum `ProbeStatus<T>` with variants `Clean`, `Debt(T)`, `Unknown`.
  - Define `DocDebtReport` struct containing:
    - `openspec_desync: ProbeStatus<Vec<OpenSpecDesyncFinding>>`
    - `stale_pending: ProbeStatus<Vec<StalePendingFinding>>`
    - `solution_drift: ProbeStatus<Vec<SolutionDriftFinding>>`
    - `git_available: bool`
  - Define finding structs (`OpenSpecDesyncFinding`, `StalePendingFinding`, `SolutionDriftFinding`).
  - Extend `RepoState` with `pub doc_debt: Option<DocDebtReport>`.
  - Implement `serde::Serialize` and `serde::Deserialize` for all new types.
  - Add unit tests in `src/commands/tests/workflow.rs` validating tri-state serialization round-trip and behavior when Git is unavailable.

### Unit 2: Workspace Config Overrides in `src/state/state.rs` & `src/state/ports.rs`
- **Files:** `src/state/state.rs`, `src/state/ports.rs`, `src/state/tests/state.rs`
- **Target:** ~110 LOC
- **Approach:**
  - Define `DocHygieneConfig` struct with fields:
    - `stale_spec_days: u32` (default: 21)
    - `check_solution_paths: bool` (default: true)
    - `require_solution_frontmatter: bool` (default: true)
  - Add `doc_hygiene: Option<DocHygieneConfig>` to `State`.
  - Update `State::merge_overrides` to merge `doc_hygiene` from workspace `.ce-ai.json`.
  - Add unit tests in `src/state/tests/state.rs` verifying `.ce-ai.json` overrides merge cleanly and default values apply when omitted.

### Unit 3: Probe 1 (Git-Aware Desync) & Probe 2 (Stale Inactivity Watchdog)
- **Files:** `src/commands/workflow.rs`, `src/commands/tests/workflow.rs`
- **Target:** ~190 LOC
- **Approach:**
  - Implement `probe_openspec_desync(repo_root: &Path, git_available: bool) -> ProbeStatus<Vec<OpenSpecDesyncFinding>>`:
    - Reads `openspec/changes/*` excluding `archive`.
    - Detects parent tasks `[x]` with subtasks `[ ]`.
    - If Git is available, checks `git log -n 1 -- <path>` or commit landing on `HEAD`.
    - Checks if `tasks.md` references a version tag surpassed in `Cargo.toml`.
    - When substrate is missing and evaluation is indeterminate, returns `ProbeStatus::Unknown`.
  - Implement `probe_stale_pending_openspecs(repo_root: &Path, stale_days: u32, git_available: bool) -> ProbeStatus<Vec<StalePendingFinding>>`:
    - Checks elapsed days from last commit touching change folder.
    - Falls back to filesystem `mtime` in No-Git environments.
    - If elapsed days > `stale_days` and tasks are incomplete, flags `StalePending`.
  - Add unit tests in `src/commands/tests/workflow.rs` testing fixtures for stranded features, stale features, and No-Git fallbacks.

### Unit 4: Probe 5 (Solution Library Drift Probe & YAML Linter)
- **Files:** `src/commands/workflow.rs`, `src/commands/tests/workflow.rs`
- **Target:** ~150 LOC
- **Approach:**
  - Implement `probe_solution_drift(repo_root: &Path, config: &DocHygieneConfig) -> ProbeStatus<Vec<SolutionDriftFinding>>`:
    - Scans `docs/solutions/**/*.md`.
    - Parses YAML frontmatter between `---` boundaries and asserts required keys: `title`, `category` (or `module`), `problem_type`, `tags`, `applies_when`.
    - Extracts backticked repo file paths matching `src/**/*.rs` or `tests/**/*.rs` and verifies path existence in `repo_root`. Ignores external URLs and general tokens.
  - Add unit tests in `src/commands/tests/workflow.rs` verifying dead file path detection and frontmatter schema validation.

### Unit 5: Turn-0 Delivery & Doctor Integration
- **Files:** `src/commands/workflow.rs`, `src/commands/doctor.rs`, `src/commands/tests/doctor.rs`, `tests/cli.rs`
- **Target:** ~180 LOC
- **Approach:**
  - Wire `probe_doc_debt` into `probe_repo_state`.
  - Format single-line Turn-0 summary in `workflow resume` and `workflow status`:
    - Clean: `doc debt: clean` (or omitted).
    - Debt with Git: `doc debt: 2 unarchived (code merged), 1 stale spec (34d), 2 dead solution links`.
    - Debt without Git: `doc debt: 1 stale spec [git: n/a], 2 dead solution links`.
  - In `doctor::run` in `src/commands/doctor.rs`, emit non-blocking `doctor-warn:` lines with copy-pasteable CLI commands:
    - Desynced: `doctor-warn: openspec change '<feature>' is merged or complete with open subtasks — run 'ce-ai archive <feature> --auto-mark' or 'ce-ai archive <feature> --status "..."'`
    - Stale: `doctor-warn: openspec change '<feature>' has been pending for X days with no activity — resume, shelve, or archive with '--status "abandoned/superseded"'`
    - Solution drift: `doctor-warn: solution '<rel_path>' references non-existent path '<dead_path>'`
  - Integration tests in `tests/cli.rs` testing CLI execution with and without Git substrate.

### Unit 6: Pedagogical Guide & README Updates
- **Files:** `docs/user-guide/doc-hygiene-and-debt-explained.md`, `README.md`, `CONCEPTS.md`, `CHANGELOG.md`
- **Target:** ~150 LOC
- **Approach:**
  - Author `docs/user-guide/doc-hygiene-and-debt-explained.md` as a Diátaxis Explanation guide for Beginners in an engaging "Professor's Walkthrough" tone.
  - Update `README.md` to explain the what & why of doc debt for newcomers, and add the new guide to the Documentation Map table, while strictly verifying `README.md <= 100` lines.
  - Update `CONCEPTS.md` and `CHANGELOG.md`.

## Quality & Performance Gates

1. **Sub-15ms Turn-0 Performance:** In-memory benchmark validates that `probe_doc_debt` runs in under 15ms.
2. **Substrate Hermeticity:** Tests executing in non-git directories assert `[git: n/a]` / `Unknown` rather than false `Clean` reports.
3. **Hard-Gate Invariants:**
   - Zero Clippy warnings: `cargo clippy --all-targets --all-features -- -D warnings`.
   - Strict formatting: `cargo fmt --check`.
   - 100% tests pass: `cargo test`.
   - `README.md` line count: `wc -l README.md` <= 100 lines.
