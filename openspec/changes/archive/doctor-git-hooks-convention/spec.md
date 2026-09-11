# Specification: Git-Hooks Probe Adoption Guard

## Formal Requirements

### Requirement 1: Non-Adopted Repositories With Custom Hooks
- **WHEN** a repository has `core.hooksPath` set to a path other than `.githooks` (such as `.husky/_`) AND no `.githooks/` directory exists at the repository root
- **THEN** `ce-ai doctor` SHALL NOT record a finding for `git-hooks: core.hooksPath`
- **AND** `ce-ai doctor` SHALL output an informational message: `doctor-info: git-hooks core.hooksPath set to '<val>' (not the .githooks convention; skipping)`.

### Requirement 2: Adopted Repositories With Drifted HooksPath
- **WHEN** a repository has a `.githooks/` directory at the repository root AND `core.hooksPath` is set to a path other than `.githooks`
- **THEN** `ce-ai doctor` SHALL record a finding: `git-hooks: core.hooksPath set to '<val>', expected '.githooks'`.

### Requirement 3: Adopted Repositories With Missing Pre-Commit Hook
- **WHEN** `core.hooksPath` points to `.githooks` AND `.githooks/pre-commit` does not exist
- **THEN** `ce-ai doctor` SHALL record a finding: `git-hooks: .githooks/pre-commit missing`.

### Requirement 4: Repositories Without Configured HooksPath
- **WHEN** `core.hooksPath` is not set in git config
- **THEN** `ce-ai doctor` SHALL output: `doctor-info: git-hooks core.hooksPath not set` without recording a finding.
