# Specification: Documentation Technical Debt Diagnostic Engine & Probes

## Scenario 1: Substrate Availability & Tri-State Evaluation
- **WHEN** any documentation probe runs in a repository with `.git` and encounters no debt:
  - **THEN** it returns `ProbeStatus::Clean`.
- **WHEN** a probe evaluates a check requiring Git history in a non-git directory:
  - **THEN** it returns `ProbeStatus::Unknown`.
  - **AND** under no circumstance does it return `ProbeStatus::Clean`.
- **WHEN** a probe detects one or more findings:
  - **THEN** it returns `ProbeStatus::Debt(findings)`.

## Scenario 2: Probe 1 — OpenSpec Desync & Merged Code Detection
- **WHEN** an OpenSpec change folder has 100% of top-level tasks marked `[x]` but one or more subtasks are `[ ]`:
  - **THEN** `probe_openspec_desync` returns `ProbeStatus::Debt` containing `OpenSpecDesyncFinding` with `DesyncReason::ParentTasksCompleteSubtasksOpen`.
- **WHEN** an OpenSpec change has unchecked tasks but git commits modifying the feature or mentioning its name are found in `HEAD`:
  - **THEN** `probe_openspec_desync` returns `ProbeStatus::Debt` with `DesyncReason::CodeMergedToMain`.
- **WHEN** an OpenSpec change tasks cite a version tag lower than the version declared in `Cargo.toml`:
  - **THEN** `probe_openspec_desync` returns `ProbeStatus::Debt` with `DesyncReason::ReleaseVersionSurpassed`.
- **WHEN** no git repository exists and tasks are incomplete with non-complete top-level tasks:
  - **THEN** Git-dependent checks return `ProbeStatus::Unknown`.

## Scenario 3: Probe 2 — Stale Pending Change Watchdog
- **WHEN** an OpenSpec change has incomplete tasks and its last git commit was more than `stale_spec_days` ago:
  - **THEN** `probe_stale_pending_openspecs` returns `ProbeStatus::Debt` containing `StalePendingFinding` with `InactivitySource::GitCommitDate`.
- **WHEN** no git repository exists:
  - **THEN** the probe calculates elapsed days using filesystem `mtime` and reports `InactivitySource::FilesystemMtime`.
- **WHEN** an incomplete change has had commits or modifications within `stale_spec_days`:
  - **THEN** it returns `ProbeStatus::Clean`.

## Scenario 4: Probe 5 — Solution Library Drift Detection
- **WHEN** a markdown file in `docs/solutions/**/*.md` references a backticked path matching `src/**` or `tests/**` that does not exist in the working directory:
  - **THEN** `probe_solution_drift` returns `ProbeStatus::Debt` containing the dead path in `SolutionDriftFinding.dead_paths`.
- **WHEN** a markdown file in `docs/solutions/` lacks any of `title`, `category` (or `module`), `problem_type`, `tags`, or `applies_when`:
  - **THEN** `probe_solution_drift` returns `ProbeStatus::Debt` listing missing fields in `SolutionDriftFinding.missing_frontmatter_fields`.
- **WHEN** all solution files exist, contain valid frontmatter, and cite valid paths:
  - **THEN** it returns `ProbeStatus::Clean`.

## Scenario 5: Turn-0 Delivery & Doctor Output
- **WHEN** `ce-ai workflow resume` or `status` is executed:
  - **THEN** a single-line summary of `DocDebtReport` is rendered in Turn-0 `RepoState`.
  - **AND** if Git is unavailable, `[git: n/a]` is displayed alongside any available filesystem findings.
- **WHEN** `ce-ai doctor` is executed:
  - **THEN** non-blocking `doctor-warn:` lines are emitted for each detected debt item with copy-pasteable CLI commands.
  - **AND** `doctor` exits with code `0` unless `--strict` mode is enabled.

## Scenario 6: Pedagogical Guide & README Invariants
- **WHEN** `docs/user-guide/doc-hygiene-and-debt-explained.md` is authored:
  - **THEN** it strictly belongs to the Diátaxis Explanation quadrant and is labeled for `Beginner` audience.
- **WHEN** `README.md` is updated:
  - **THEN** `wc -l README.md` is guaranteed to be <= 100 lines.
