# Specification: Claude Code Native Marketplace Plugin Divergence

## Requirements Matrix

### REQ-1: Native Marketplace Discovery & File Absence
- **WHEN** `<claude_dir>/plugins/installed_plugins.json` does not exist or cannot be read
- **THEN** `check_claude_marketplace_divergence` returns an empty vector, and `ce-ai doctor` proceeds without error.

### REQ-2: Harness Installation Guard
- **WHEN** the `claude` harness is not registered in `state.installed_harnesses` (or its workspace scope does not apply to `cwd`)
- **THEN** `check_claude_marketplace_divergence` returns an empty vector.

### REQ-3: Scope Applicability
- **WHEN** an entry in `installed_plugins.json` has `scope: "user"`
- **THEN** it is evaluated as applicable regardless of `cwd`.
- **WHEN** an entry in `installed_plugins.json` has `scope: "project"` or `scope: "local"`
- **THEN** it is evaluated as applicable if and only if `projectPath` is specified and matches or encompasses `cwd` (or `cwd` is within `projectPath`).
- **WHEN** an entry in `installed_plugins.json` has a `projectPath` that does not match `cwd`
- **THEN** that entry is ignored.

### REQ-4: Version Normalization & Equality
- **WHEN** both the native plugin entry and `ce-ai` have the same semantic version after stripping `compound-engineering-`, `compound-engineering@`, and `v` prefixes (e.g. `3.24.0` vs `compound-engineering-v3.24.0`)
- **THEN** no divergence is reported for that entry.

### REQ-5: Divergence Reporting
- **WHEN** an applicable native plugin entry has a version differing from `ce-ai`'s managed version
- **THEN** `ce-ai doctor` emits a non-blocking `doctor-info:` message in the format:
  ```
  doctor-info: claude native plugin marketplace divergence detected for '<plugin_id>' (scope: <scope>): native marketplace has v<native_version> but ce-ai managed harness is v<ce_version> (run '<update_cmd>' to update)
  ```
- **THEN** the update command specifies `claude plugin marketplace update <marketplace> && claude plugin update <plugin_id>` (where `<marketplace>` is parsed from `<plugin>@<marketplace>` or defaults to `compound-engineering-plugin`).

### REQ-6: Non-Blocking Exit Code & Read-Only Invariant
- **WHEN** one or more divergences are detected
- **THEN** the findings are NOT added to doctor's fatal `findings` list and do not cause `ce-ai doctor` to return a non-zero exit code.
- **THEN** no files under `<claude_dir>/plugins/` are modified, created, or deleted.

### REQ-7: Resilient Deserialization
- **WHEN** `installed_plugins.json` contains malformed JSON, unexpected types, or missing fields
- **THEN** the probe degrades gracefully, returning an empty vector without panicking or throwing an error.
