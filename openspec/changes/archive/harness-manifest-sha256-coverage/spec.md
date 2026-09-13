# Specification: SHA256 Manifest Coverage for Non-OpenCode Harnesses

## Requirements Matrix

### REQ-1: Sync Root Cause — Real Digests for Registration Harnesses
- **WHEN** `ce-ai sync` processes a table-driven harness (claude, kimi, codex, copilot, cursor, grok, agy, fx) whose `<config_dir>/compound-engineering/install-manifest.json` already exists
- **THEN** the rewritten manifest's `files` contains one `{path, sha256}` entry per file physically present under the managed dir (excluding `install-manifest.json` itself), reflecting actual on-disk bytes.
- **THEN** the prior manifest's `installed_at` and `config_mutations` are preserved.

### REQ-2: No Manifest Fabrication
- **WHEN** a table-driven harness has no pre-existing manifest at its managed dir
- **THEN** `sync` does not create one (behavior unchanged).

### REQ-3: Error Propagation
- **WHEN** the registration-arm manifest write fails (I/O, serialization)
- **THEN** `sync` returns the error instead of silently continuing.

### REQ-4: Harvest Helper
- **WHEN** `InstallManifest::harvest(managed_dir)` is invoked
- **THEN** it returns SHA256 entries for all regular files under `managed_dir`, recursively, with paths relative to `managed_dir`, sorted deterministically, excluding `install-manifest.json`.
- **WHEN** `managed_dir` does not exist
- **THEN** it returns an empty vector without error.

### REQ-5: Workflow Drift Count Coverage
- **WHEN** `probe_manifest_drift_count` runs and the claude and/or kimi managed manifest exists with non-empty `files`
- **THEN** drift actions from those manifests (diffed against their own managed dirs) are included in the returned count, in addition to OpenCode drift.
- **WHEN** the claude/kimi manifest is missing or has empty `files`
- **THEN** that harness contributes zero to the count (graceful degradation, no error).

### REQ-6: Doctor Diff Findings Coverage
- **WHEN** the doctor diff probe detects drift from the claude or kimi manifest
- **THEN** a finding `diff: <harness> <kind> <path>` is pushed (e.g. `diff: claude modified plugins/compound-engineering.js`).
- **WHEN** the claude/kimi manifest is absent
- **THEN** no finding is produced for that harness and doctor does not error.

### REQ-7: Status Output Compatibility
- **WHEN** only the OpenCode manifest exists and there is no drift
- **THEN** `ce-ai status` prints exactly `drift: none`.
- **WHEN** no manifest exists at all
- **THEN** `ce-ai status` prints exactly `drift: unknown (no install manifest)`.
- **WHEN** claude or kimi manifests exist with drift
- **THEN** `ce-ai status` additionally prints `drift: <harness>: <kind> <path>` lines for those harnesses.

### REQ-8: Hermetic Tests
- **WHEN** the regression tests for this change run
- **THEN** they operate exclusively on temporary directories (tempfile) and never read from the real `~/.claude`, `~/.kimi-code`, or `~/.config/opencode`.

### REQ-9: OpenCode Behavior Unchanged
- **WHEN** this change is applied
- **THEN** OpenCode manifest writing, diff output strings, and drift counts remain byte-identical for all existing OpenCode-only scenarios (existing tests must pass unmodified).
