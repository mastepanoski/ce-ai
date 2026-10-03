# Requirements Specification: Phase 2 — CE Compatibility Layer & Fleet Subsystem

## Requirements & Acceptance Criteria

### REQ-1: Centralized CE Compatibility Layer (`src/compat/`)
- **REQ-1.1**: WHEN `CeDocsConfig::discover(repo_root)` is called, THEN it MUST check for `.compound-engineering/config.yaml` or `.local.yaml`, parse `docs_root` if present, or default to `docs/`.
- **REQ-1.2**: WHEN `plans_dir()` or `solutions_dir()` is queried on `CeDocsConfig`, THEN it MUST return `<repo_root>/<docs_root>/plans` and `<repo_root>/<docs_root>/solutions` respectively.
- **REQ-1.3**: WHEN `get_canonical_skill_contract(name)` is called, THEN it MUST report whether `mode:return-to-caller` is supported for that skill (specifically `ce-work` and `ce-resolve-pr-feedback`).

### REQ-2: Upstream Solution Frontmatter Schema Conformance (`src/compat/schema.rs`)
- **REQ-2.1**: WHEN parsing a solution Markdown document, THEN `CeSolutionFrontmatter` MUST require `module`, `date`, `problem_type`, `component`, and `severity`.
- **REQ-2.2**: WHEN parsing a solution Markdown document, THEN `tags` and `applies_when` MUST be optional fields.
- **REQ-2.3**: WHEN `is_bug_track()` is called on a solution frontmatter, THEN it MUST return true if `problem_type` is `"bugfix"` or `"bug"`.

### REQ-3: Fleet Version Pinning (`ce-ai fleet pin <version>`)
- **REQ-3.1**: WHEN `ce-ai fleet pin vX.Y.Z` is executed, THEN `vX.Y.Z` MUST be atomically persisted to `state.json` under `fleet.pinned_version`.
- **REQ-3.2**: WHEN `ce-ai fleet pin` is executed with an empty or invalid version, THEN it MUST return a `CeError::UsageError` (exit code 2).

### REQ-4: Fleet Status Auditing (`ce-ai fleet status`)
- **REQ-4.1**: WHEN `ce-ai fleet status` is executed, THEN it MUST iterate across all detected or installed harnesses and report their installed CE version alongside the pinned version.
- **REQ-4.2**: WHEN `--json` is supplied to `ce-ai fleet status`, THEN it MUST output a valid JSON object matching `FleetStatusReport`.
- **REQ-4.3**: WHEN any installed harness has a different version than the pinned version, THEN the alignment status for that harness MUST be reported as `Divergent`.

### REQ-5: Fleet Sync Plan & Execution (`ce-ai fleet sync`)
- **REQ-5.1**: WHEN `ce-ai fleet sync --dry-run` is executed, THEN it MUST display the planned actions (e.g. CLI commands or config mutations) without executing them.
- **REQ-5.2**: WHEN `ce-ai fleet sync` is executed without `--dry-run`, THEN it MUST execute the planned actions and update `state.fleet.last_sync`.
- **REQ-5.3**: WHEN no version is pinned in `state.json`, THEN `ce-ai fleet sync` MUST instruct the user to run `ce-ai fleet pin <version>` first and exit with code 2.

### REQ-6: Refactoring Existing Code to Use `src/compat/`
- **REQ-6.1**: WHEN `src/commands/doc.rs` and `src/commands/workflow.rs` perform docs root or frontmatter checks, THEN they MUST use `src/compat/` instead of duplicating inline logic.
