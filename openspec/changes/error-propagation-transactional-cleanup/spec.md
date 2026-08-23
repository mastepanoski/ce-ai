# OpenSpec Requirements: Error Propagation & Transactional State Commit

- **Change:** `error-propagation-transactional-cleanup`
- **Issue:** #162 (P1)

---

## 📜 Acceptance Criteria

### Requirement 1: Error Propagation on Required Operations
- **WHEN** a required filesystem operation (file removal, backup restoration, atomic write, `.gitignore` update) fails in `uninstall`, `deinit-prj`, or `init-prj`
- **THEN** the command SHALL immediately return a non-zero exit code (`CeError::IO` / `CeError::Runtime`)
- **AND** SHALL NOT print a success completion message.

### Requirement 2: Transactional State Persistence
- **WHEN** a required operation fails during `uninstall`, `deinit-prj`, or `init-prj`
- **THEN** `state.json` SHALL NOT be saved to reflect completion of the failed operation.

### Requirement 3: Best-Effort Non-Critical Cleanup Warnings
- **WHEN** a best-effort non-critical cleanup (e.g. skill registry index update) fails
- **THEN** `ce-ai` SHALL log a `warning:` to `stderr` unless `--quiet` is specified
- **AND** SHALL proceed without failing the overall command.
