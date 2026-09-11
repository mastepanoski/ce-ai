# Specification: Zero-Step Environment Drift Recovery via Live `RepoState` Sync

## 1. Functional Requirements

### Scenario 1: Clean Working Tree Turn Resumption
- **WHEN** a user or agent invokes `ce-ai workflow resume` in a clean repository on branch `main` with valid manifest and no uncommitted files:
- **THEN** `ce-ai` MUST output `== [Environment State & Drift Status] ==` reporting:
  - `git branch: main`
  - `working tree: clean (0 uncommitted changes)`
  - `manifest integrity: clean (0 drifted files)`
  - `adoption block: ok` (derived via `check_adoption_block_status`)
- **AND** the command MUST exit with code `0`.

### Scenario 2: External File Edits & Branch Drift Detection
- **WHEN** a user modifies files or checks out a different git branch outside the agent turn:
- **THEN** upon invoking `ce-ai workflow resume`:
  - `ce-ai` MUST reflect the newly checked-out git branch in `git_branch`.
  - `ce-ai` MUST list all modified, untracked, or deleted paths under `working tree: X modified files`.
  - `is_git_clean` MUST evaluate to `false`.
- **AND** the agent receives exact ground-truth state in Turn 0.

### Scenario 3: Managed Plugin Manifest Drift
- **WHEN** files inside `~/.config/opencode/compound-engineering` have been altered or deleted outside `ce-ai`:
- **THEN** `ce-ai workflow resume` MUST:
  - Report `manifest_drift_count > 0`.
  - Display `! Warning: Drift detected in X managed files. Run 'ce-ai sync' to reconcile.`.
  - NOT fail or exit with a non-zero exit code during normal workflow resumption.

### Scenario 4: Non-Git or Sandbox Environments
- **WHEN** `ce-ai workflow resume` is executed in a directory that is not a git repository or where the `git` binary is unavailable:
- **THEN** `ce-ai` MUST:
  - Gracefully set `git_branch: None`, `head_sha: None`, and `is_git_clean: true`.
  - Proceed with workflow and manifest status display without crashing or returning an IO error.

### Scenario 5: Machine-Readable JSON Output
- **WHEN** `ce-ai workflow resume --json` is executed:
- **THEN** the JSON payload MUST contain a top-level `"repo_state"` object conforming to the `RepoState` schema:
  - `git_branch`: `string | null`
  - `head_sha`: `string | null`
  - `is_git_clean`: `boolean`
  - `modified_files`: `string[]`
  - `manifest_drift_count`: `number`
  - `adoption_status`: `string | null`
  - `openspec_context`: `OpenSpecContextInfo | null`

## 2. Non-Functional & Quality Constraints

- **Latency:** Probing `RepoState` MUST complete in under 15ms on local filesystems.
- **Determinism:** Cryptographic integrity MUST rely exclusively on SHA256 hashing. Timestamps MUST NOT be used to declare a file clean or drifted.
- **SSOT Reuse:** Adoption status classification MUST call `src/commands/init_prj.rs::check_adoption_block_status` rather than duplicating hash calculation.
- **Backwards Compatibility:** Existing fields in `workflow resume --json` (`"workflow"` and `"openspec_context"`) MUST remain present and unchanged.
