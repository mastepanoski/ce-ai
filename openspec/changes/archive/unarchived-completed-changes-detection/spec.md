# Specification: Deterministic Detection of Unarchived Completed OpenSpec Changes

## Requirements

### Requirement 1: Shared Task Checkbox Counting & Degradation
- **WHEN** `count_task_checkboxes` is called with a path to a `tasks.md` file,
- **THEN** it MUST parse each line, counting lines starting with `- [x]` or `- [X]` as completed tasks and lines starting with `- [ ]` as incomplete tasks, returning `(completed_tasks, total_tasks)`.
- **WHEN** the given path does not exist, cannot be read, or is a directory,
- **THEN** it MUST return `(0, 0)` without panicking or bubbling errors.

### Requirement 2: Repo-Wide Unarchived Completed Changes Discovery
- **WHEN** `probe_unarchived_completed_changes` is invoked on a repository root,
- **THEN** it MUST iterate through all direct subdirectories under `openspec/changes/`, ignoring the directory literally named `archive`.
- **WHEN** a subdirectory contains a `tasks.md` file with `total_tasks > 0` and `completed_tasks == total_tasks`,
- **THEN** it MUST be included in the returned collection as an `UnarchivedChange` struct containing the feature directory name, completed tasks count, and total tasks count.
- **WHEN** any I/O operation fails during directory or file inspection,
- **THEN** the probe MUST skip that entry gracefully without aborting the rest of the scan.
- **THEN** the returned collection MUST be deterministically sorted alphabetically by feature name.

### Requirement 3: `RepoState` Data Model Extension
- **WHEN** `probe_repo_state` is executed,
- **THEN** it MUST invoke `probe_unarchived_completed_changes` and assign the result to `repo_state.unarchived_completed_changes`.
- **WHEN** serialized to JSON (e.g. in `ce-ai workflow resume --json`),
- **THEN** `unarchived_completed_changes` MUST be included as an array of structured objects when non-empty, and omitted when empty.

### Requirement 4: Turn-0 `resume_lines` Compact Summary
- **WHEN** `resume_lines` renders the `"== [Environment State & Drift Status] =="` block,
- **THEN** it MUST include an `openspec ledger:` summary line outside the active feature context block.
- **WHEN** `unarchived_completed_changes` is empty,
- **THEN** it MUST output `  openspec ledger: clean (0 pending archival)`.
- **WHEN** `unarchived_completed_changes` contains $N > 0$ items,
- **THEN** it MUST output `  openspec ledger: ! N change(s) complete but not archived — run 'ce-ai doctor' for details`.

### Requirement 5: `status_lines` & `checkpoint_lines` Non-Blocking Warnings
- **WHEN** `status_lines` or `checkpoint_lines` is rendered and `unarchived_completed_changes` is non-empty,
- **THEN** it MUST append `! Warning: N OpenSpec change(s) complete but not archived — run 'ce-ai doctor' for details`.
- **WHEN** `unarchived_completed_changes` is empty,
- **THEN** no unarchived completed changes warning line MUST be emitted.

### Requirement 6: `ce-ai doctor` Verbose Diagnostic Warnings
- **WHEN** `ce-ai doctor` runs on a repository with unarchived completed changes,
- **THEN** it MUST output a warning for each completed change matching:
  `doctor-warn: openspec change '<feature>' is complete (N/N tasks) but not archived — see openspec/changes/archive/README.md`.
- **THEN** these warnings MUST NOT be added to `findings`, and `ce-ai doctor` MUST exit with code 0 in the absence of other fatal findings.
