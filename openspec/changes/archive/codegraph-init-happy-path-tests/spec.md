# Spec: CodeGraph Subprocess Execution Happy Path Coverage

## Requirements

### Requirement 1: `tools init codegraph` Subprocess Happy Path
- **WHEN** user executes `ce-ai tools init codegraph` on a directory where `.codegraph/` does not exist and `codegraph` executable is found in `PATH`:
  - **THEN** `ce-ai` executes `codegraph init <dir>`, exits `0`, confirms initialization on stdout, and `.codegraph/` exists on disk.

### Requirement 2: `init-prj` Auto-Initialization Happy Path
- **WHEN** user executes `ce-ai init-prj` on a git repository where `.codegraph/` does not exist and `codegraph` executable is found in `PATH`:
  - **THEN** `ce-ai init-prj` adopts the project, invokes `init_codegraph_if_available`, creates `.codegraph/` on disk, reports `✓ Initialized CodeGraph index (.codegraph/)`, and exits `0`.
