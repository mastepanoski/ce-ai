# Spec: CodeGraph Native Init Support & gentle-ai Residual Cleanup

## Requirements

### Requirement 1: `ce-ai tools init codegraph`
- **WHEN** user runs `ce-ai tools init codegraph` in a project where `codegraph` is in PATH and `.codegraph/` does not exist:
  - **THEN** `ce-ai` runs `codegraph init` targeting the project directory and outputs confirmation.
- **WHEN** user runs `ce-ai tools init codegraph` and `.codegraph/` already exists:
  - **THEN** `ce-ai` reports that the index is already initialized and exits `0`.
- **WHEN** user runs `ce-ai tools init codegraph` and `codegraph` is not in `PATH`:
  - **THEN** `ce-ai` returns exit code `2` (`CeError::Usage`) explaining that the `codegraph` binary is missing and giving installation instructions.
- **WHEN** user runs `ce-ai tools init unsupported-tool`:
  - **THEN** `ce-ai` returns exit code `2` (`CeError::Usage`) indicating only `codegraph` is currently supported for `init`.
- **WHEN** user runs `ce-ai tools init codegraph --dry-run`:
  - **THEN** `ce-ai` prints `[dry-run] would run 'codegraph init'` without executing subprocess or mutating filesystem.

### Requirement 2: Auto-Init in `ce-ai init-prj`
- **WHEN** user runs `ce-ai init-prj` on a repository, `codegraph` is installed on PATH, and `.codegraph/` does not exist:
  - **THEN** `init-prj` executes `codegraph init` for the adopted directory.
- **WHEN** `codegraph init` fails during `ce-ai init-prj`:
  - **THEN** `init-prj` emits a warning and continues adoption without erroring (non-fatal sidecar initialization).

### Requirement 3: Cleanup of gentle-ai References
- **WHEN** `ce-ai audit` encounters an uninitialized `.codegraph/` directory:
  - **THEN** it reports `detail: ".codegraph/ index not initialized (run 'codegraph init' or 'ce-ai tools init codegraph')"` without any reference to `gentle-ai`.
- **WHEN** `ce-ai doctor` logs missing `.codegraph/`:
  - **THEN** it suggests `'ce-ai tools init codegraph'`.
- **WHEN** reading `docs/user-guide/quick-start-workflow-guide.md`:
  - **THEN** instructions state `codegraph init` or `ce-ai tools init codegraph`.
- **WHEN** reading `openspec/changes/workspace-scoped-workflow-and-gitignore/exploration.md`:
  - **THEN** code snippets use `# BEGIN CE-AI MANAGED BLOCK` instead of `gentle-ai:ce-ai-ignore`.
