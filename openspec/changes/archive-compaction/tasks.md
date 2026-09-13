# Tasks: Generational Archive Compaction & Milestone Rollups

Work-unit changed-line estimates total: ~1010 LOC (~200 LOC/work-unit target across code-bearing units; tests/docs are additive verification).

- [x] **Work Unit 1: Configuration & CLI Argument Plumbing** (~150 LOC)
  - [x] 1.1 Add `archive_compaction_threshold: u32` to `DocHygieneConfig` in `src/state/state.rs` with default `30` and JSON deserialization.
  - [x] 1.2 Define `ArchiveSubcommand::Compact(CompactArgs)` and `CompactArgs` in `src/commands/workflow.rs` (`--before`, `--milestone`, `--threshold`, `--dry-run`, `--no-tarball`, `--keep-loose`).
  - [x] 1.3 Plumb `CompactArgs` in `workflow::run_archive` and dispatch to `run_archive_compact`.
  - [x] 1.4 (TDD) In `src/state/tests/state.rs` and `src/commands/tests/workflow.rs`, verify CLI argument parsing and config threshold loading.

- [x] **Work Unit 2: Candidate Resolution, Date Extraction & Milestone Grouping** (~190 LOC)
  - [x] 2.1 Define `CompactionCandidate` and `MilestonePlan` structs in `src/commands/workflow.rs`.
  - [x] 2.2 Implement multi-tier date extractor: folder prefix `YYYY-MM-DD-` -> `git log -1` -> filesystem `mtime`.
  - [x] 2.3 Implement candidate discovery in `openspec/changes/archive/` (ignoring `milestones/` and `README.md`).
  - [x] 2.4 Implement milestone grouping logic: calendar quarter (`YYYY-QX`) or custom `--milestone`.
  - [x] 2.5 (TDD) In `src/commands/tests/workflow.rs`, test date extraction tiers, candidate filtering with `--before`, and milestone grouping.

- [x] **Work Unit 3: Rollup Markdown Generation, Tarball Packaging & Safe Pruning** (~220 LOC)
  - [x] 3.1 Implement metadata extraction from candidate packages (problem statement from `proposal.md`, acceptance criteria from `spec.md`, tasks ratio from `tasks.md`).
  - [x] 3.2 Implement `generate_milestone_markdown(milestone: &str, candidates: &[CompactionCandidate], tarball: Option<&str>) -> String`.
  - [x] 3.3 Implement `create_milestone_tarball` using `flate2` and `tar::Builder` with verified safe relative entries.
  - [x] 3.4 Implement tarball verification and safe directory removal (unless `--keep-loose` is set).
  - [x] 3.5 Implement `update_archive_readme_ledger` updating `## Compacted Milestones` table in `openspec/changes/archive/README.md`.
  - [x] 3.6 (TDD) In `src/commands/tests/workflow.rs`, test end-to-end compaction cycle, `--dry-run` non-mutation, tarball extraction safety, and README updating.

- [x] **Work Unit 4: Doctor Health Check & Turn-0 Summary Integration** (~170 LOC)
  - [x] 4.1 Define `ArchiveCompactionFinding` and add `archive_compaction: ProbeStatus<ArchiveCompactionFinding>` to `DocDebtReport`.
  - [x] 4.2 Implement `probe_archive_compaction(repo_root: &Path, config: &DocHygieneConfig) -> ProbeStatus<ArchiveCompactionFinding>`.
  - [x] 4.3 Integrate probe into `probe_doc_debt` and format `summary_line`.
  - [x] 4.4 In `src/commands/doctor.rs`, emit non-blocking `doctor-warn: archive has N uncompacted packages (>30 threshold); run 'ce-ai archive compact'` advisory.
  - [x] 4.5 (TDD) In `src/commands/tests/doctor.rs` and `src/commands/tests/workflow.rs`, test doctor warning emission and `summary_line` formatting.

- [x] **Work Unit 5: End-to-End Integration Tests & CLI Verification** (~180 LOC)
  - [x] 5.1 In `tests/cli.rs`, add CLI integration tests for `ce-ai archive compact --dry-run` and real compaction in a simulated workspace.
  - [x] 5.2 Test `ce-ai workflow archive compact` alias parity.
  - [x] 5.3 Test edge cases: empty archive, all archives below threshold, corrupted files, `--no-tarball`, `--keep-loose`.

- [x] **Work Unit 6: Documentation, README Compliance & Quality Gates** (~100 LOC)
  - [x] 6.1 Update `README.md` with `ce-ai archive compact` explanation while strictly enforcing `README.md <= 100` lines (`wc -l README.md`).
  - [x] 6.2 Update `CONCEPTS.md` with `Generational Archive Compaction` and `Milestone Rollups`.
  - [x] 6.3 Update `CHANGELOG.md` under `[Unreleased]`.
  - [x] 6.4 Execute full verification suite: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.
