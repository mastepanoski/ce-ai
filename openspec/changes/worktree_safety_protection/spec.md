# OpenSpec Specification: Worktree Safety Protection

## Requirements

### Requirement 1: Hard Invariant Preservation
- **WHEN** an AI agent reads `AGENTS.md`,
- **THEN** it MUST encounter Rule 8 in the Hard-Gate Invariant Index prohibiting automated or unconfirmed deletion of sibling worktrees.

### Requirement 2: Worktree Diagnostic Probing
- **WHEN** `ce-ai doctor` is executed within a Git repository,
- **THEN** it MUST execute `git worktree list --porcelain`, parse all registered worktree paths, filter out the primary working directory, and emit an advisory log (`doctor-info: active sibling worktree at <path>`) for each additional worktree.

### Requirement 3: Failure Isolation
- **WHEN** `git worktree list` fails or no additional worktrees exist,
- **THEN** `ce-ai doctor` MUST continue silently without raising an error finding.
