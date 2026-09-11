# Tasks: OpenSpec Change Archival CLI Command & Ledger Synchronization

## Work Unit 1: CLI Argument Definitions & Registry Dispatch (~45 LOC)
- [x] 1.1 Define `ArchiveArgs` struct in `src/commands/workflow.rs` with `feature: Option<String>`, `all: bool`, `dry_run: bool`, and `status: Option<String>`.
- [x] 1.2 Add `Action::Archive(ArchiveArgs)` to `Action` enum in `src/commands/workflow.rs`.
- [x] 1.3 Add `Commands::Archive(crate::commands::workflow::ArchiveArgs)` to `Commands` enum in `src/commands/registry.rs` and dispatch to `workflow::run_archive`.
- [x] 1.4 Wire `Action::Archive(args)` in `src/commands/workflow.rs::run` to `run_archive`.

## Work Unit 2: Core Validation & Safe Mover Implementation (~95 LOC)
- [x] 2.1 Define `ArchiveCriterion` and `ArchiveOutcome` structs in `src/commands/workflow.rs`.
- [x] 2.2 Implement `validate_and_archive_feature(repo_root: &Path, feature: &str, status: Option<&str>, dry_run: bool) -> Result<ArchiveOutcome, CeError>`:
  - Verify `openspec/changes/<feature>` exists and is a directory (`CeError::Usage`).
  - Assert `openspec/changes/archive/<feature>` does not exist (`CeError::State`).
  - Evaluate criteria: Criterion 1 (mechanical 100% `[x]`) vs Criterion 2 (`--status "<evidence>"`).
  - Verify feature directory git status (fail-closed if uncommitted non-spec edits exist).
  - If Criterion 2, prepend `> STATUS: <status>\n\n` to `tasks.md`.
  - Execute move: `git mv` with atomic directory rename + git stage fallback.
- [x] 2.3 Implement `archive_all_completed_changes(repo_root: &Path, dry_run: bool) -> Result<Vec<ArchiveOutcome>, CeError>` using `probe_unarchived_completed_changes`.

## Work Unit 3: Ledger Sync & State Reconciliation (~55 LOC)
- [x] 3.1 Implement `sync_archive_readme_ledger(repo_root: &Path, outcomes: &[ArchiveOutcome], dry_run: bool) -> Result<(), CeError>` appending audit sweep records to `openspec/changes/archive/README.md`.
- [x] 3.2 Implement `reconcile_state_active_feature(ctx: &Context, archived_features: &[String], dry_run: bool) -> Result<(), CeError>` clearing active feature in `state.json` if archived.
- [x] 3.3 Implement `run_archive(ctx: &Context, args: &ArchiveArgs) -> Result<(), CeError>` coordinating validation, move, ledger update, and active feature clearing.

## Work Unit 4: Unit Tests & Edge Case Coverage (~80 LOC)
- [x] 4.1 In `src/commands/tests/workflow.rs`, add unit tests:
  - `test_archive_feature_criterion_1_success`
  - `test_archive_feature_criterion_2_status_attestation`
  - `test_archive_feature_incomplete_tasks_rejected`
  - `test_archive_feature_collision_prevention`
  - `test_archive_feature_dry_run_no_mutations`
  - `test_archive_all_completed_sweep`

## Work Unit 5: CLI Integration Tests in tests/cli.rs (~80 LOC)
- [x] 5.1 Implement integration tests in `tests/cli.rs`:
  - `cli_workflow_archive_single_feature`
  - `cli_top_level_archive_alias`
  - `cli_archive_all_batch`
  - `cli_archive_dry_run`
  - `cli_archive_error_exit_codes` (codes 2, 3, 6)

## Work Unit 6: Versioning, CHANGELOG & Batch Backlog Drain (~30 LOC)
- [x] 6.1 Bump SemVer to `1.52.0` in `Cargo.toml` and update `Cargo.lock`.
- [x] 6.2 Document `ce-ai archive` and `ce-ai workflow archive` in `CHANGELOG.md`.
- [x] 6.3 Execute `ce-ai archive --all` on the repository to drain the 33 unarchived changes backlog and verify `ce-ai doctor` reports `openspec ledger: clean (0 pending archival)`.
