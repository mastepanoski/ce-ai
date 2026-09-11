# Specification: OpenSpec Change Archival CLI Command & Ledger Synchronization

## Requirements

### Requirement 1: Single Feature Archival under Criterion 1 (Mechanical Completion)
- **WHEN** `ce-ai workflow archive <feature>` (or `ce-ai archive <feature>`) is executed on a feature directory where all tasks in `tasks.md` are marked `- [x]` or `- [X]`,
- **THEN** it MUST move the directory from `openspec/changes/<feature>` to `openspec/changes/archive/<feature>`.
- **THEN** it MUST emit a success message to stdout indicating the feature name, destination path, and task counts `(N/N tasks)`.
- **THEN** it MUST exit with code `0`.

### Requirement 2: Single Feature Archival under Criterion 2 (STATUS-Attested)
- **WHEN** `ce-ai workflow archive <feature> --status "<evidence>"` is executed on a feature directory with open checkboxes (`- [ ]`),
- **THEN** it MUST prepend `> STATUS: <evidence>\n\n` to `openspec/changes/<feature>/tasks.md` using atomic file writing.
- **THEN** it MUST move the directory to `openspec/changes/archive/<feature>`.
- **THEN** it MUST emit a success message indicating Criterion 2 attestation.
- **THEN** it MUST exit with code `0`.

### Requirement 3: Rejection of Incomplete Features Without Status
- **WHEN** `ce-ai workflow archive <feature>` is executed on a feature directory containing unchecked tasks (`- [ ]`) and no `--status` flag is provided,
- **THEN** it MUST NOT move or mutate the feature directory.
- **THEN** it MUST return `CeError::Verification` with exit code `6`.
- **THEN** it MUST emit an error message explaining the remaining open task count and advising the operator to complete the tasks or supply `--status`.

### Requirement 4: Destination Collision Prevention
- **WHEN** `ce-ai workflow archive <feature>` is executed and `openspec/changes/archive/<feature>` already exists on disk,
- **THEN** it MUST NOT overwrite or delete the existing archive destination.
- **THEN** it MUST return `CeError::State` with exit code `3`.

### Requirement 5: Missing Feature & Usage Resolution
- **WHEN** `ce-ai workflow archive <feature>` is executed and `openspec/changes/<feature>` does not exist or is not a directory,
- **THEN** it MUST return `CeError::Usage` with exit code `2`.
- **WHEN** `ce-ai workflow archive` is executed without a feature argument and no active feature is recorded in `state.json`,
- **THEN** it MUST return `CeError::Usage` with exit code `2` explaining that no target feature was provided or active.
- **WHEN** `ce-ai workflow archive` is executed without a feature argument but an active feature exists in `state.json`,
- **THEN** it MUST resolve and target that active feature.

### Requirement 6: Batch Remediation via `--all`
- **WHEN** `ce-ai workflow archive --all` (or `ce-ai archive --all`) is executed,
- **THEN** it MUST invoke `probe_unarchived_completed_changes` to discover all fully-completed change folders under `openspec/changes/`.
- **WHEN** completed change folders exist,
- **THEN** it MUST archive each completed folder to `openspec/changes/archive/` sequentially, ignoring any with destination collisions with a warning.
- **THEN** it MUST report the total count of successfully archived features and exit with code `0`.
- **WHEN** no completed change folders exist,
- **THEN** it MUST output that no changes are pending archival and exit with code `0`.

### Requirement 7: Dry-Run Inspection
- **WHEN** `--dry-run` is passed to `archive` (single-feature or `--all`),
- **THEN** it MUST validate task completeness, collision safety, and target resolution.
- **THEN** it MUST output the planned directory moves and ledger updates to stdout without performing any file moves, status injections, or ledger edits.
- **THEN** it MUST exit with code `0`.

### Requirement 8: Top-Level CLI Command Parity
- **WHEN** `ce-ai archive` is invoked with any supported flags (`--all`, `--dry-run`, `--status`, `[feature]`),
- **THEN** it MUST behave identically to `ce-ai workflow archive` with the same exit codes and outputs.

### Requirement 9: Ledger Synchronization in `archive/README.md`
- **WHEN** one or more features are successfully archived (and not in `--dry-run`),
- **THEN** `ce-ai` MUST append an audit record to `openspec/changes/archive/README.md` under Historical Notes citing the date, feature name(s), and task counts.

### Requirement 10: State Active Feature Clearing
- **WHEN** an archived feature matches the active workflow feature recorded in `state.json`,
- **THEN** `ce-ai` MUST atomically update `state.json` to clear the active feature pointer.
