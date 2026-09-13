# Specification: Kimi Code Native Plugin Manager Divergence

## Requirements Matrix

### REQ-1: Native Registry Discovery & File Absence
- **WHEN** `<kimi_dir>/plugins/installed.json` does not exist, is unreadable, or contains malformed JSON
- **THEN** `check_kimi_marketplace_divergence` returns an empty vector, and `ce-ai doctor` proceeds without error.

### REQ-2: Harness Installation Guard
- **WHEN** the `kimi` harness is not registered in `state.installed_harnesses`, or its `workspace` scope `target_dir` does not apply to `cwd`
- **THEN** `check_kimi_marketplace_divergence` returns an empty vector.

### REQ-3: Enabled-Plugin Filtering
- **WHEN** a native registry entry has `id` equal to `compound-engineering` or starting with `compound-engineering@`
- **THEN** it is evaluated only when `enabled` is `true`; disabled entries are ignored.

### REQ-4: Native Version Resolution
- **WHEN** an applicable native plugin entry declares a `root` directory
- **THEN** its version is resolved from `<root>/package.json` (fallback `<root>/plugin.json`) `version` field.
- **WHEN** the version cannot be resolved (missing `root`, missing/unreadable/unparseable metadata)
- **THEN** the entry is skipped without producing a divergence or an error.

### REQ-5: Version Normalization & Equality
- **WHEN** the native plugin version and ce-ai's managed version normalize equal after stripping `compound-engineering-`, `compound-engineering@`, and `v` prefixes (e.g. `3.14.3` vs `compound-engineering-v3.14.3`)
- **THEN** no divergence is reported.

### REQ-6: Divergence Reporting
- **WHEN** an applicable, enabled native plugin entry has a version differing from ce-ai's managed version
- **THEN** `ce-ai doctor` emits a non-blocking `doctor-info:` message identifying the plugin id, both normalized versions, and advisory remediation (update via Kimi's native plugin manager, or uninstall the native plugin to use the ce-ai managed tree exclusively).

### REQ-7: Orphan Managed Tree Detection
- **WHEN** `<kimi_dir>/compound-engineering/install-manifest.json` exists (a genuine ce-ai managed tree) AND `<kimi_dir>/config.toml` does not reference that tree in `extra_skill_dirs` (missing file, missing key, or non-matching entries)
- **THEN** `check_kimi_orphan_managed_tree` returns `Some(KimiOrphanManagedTree)`, and `ce-ai doctor` emits a non-blocking `doctor-warn:` message naming the orphan tree and its remediation options.
- **WHEN** the managed tree does not exist, or `extra_skill_dirs` references it
- **THEN** no orphan finding is produced.

### REQ-8: Non-Blocking Exit Code & Read-Only Invariant
- **WHEN** one or more divergences or an orphan tree is detected
- **THEN** neither is added to doctor's fatal `findings` list and `ce-ai doctor` retains its exit behavior.
- **THEN** no files under `<kimi_dir>/` are modified, created, or deleted by these probes.

### REQ-9: Resilient Deserialization
- **WHEN** any inspected file (`installed.json`, `package.json`, `plugin.json`, `config.toml`) contains malformed data or unexpected types
- **THEN** the probes degrade gracefully, returning empty/None results without panicking or emitting error diagnostics.

### REQ-10: Hermetic Test Fixtures
- **WHEN** the unit tests for these probes run
- **THEN** they operate exclusively on temporary directories and never read from or write to the real `~/.kimi-code` home directory.
