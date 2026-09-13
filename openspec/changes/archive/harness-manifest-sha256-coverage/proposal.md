# Proposal: SHA256 Manifest Coverage for Non-OpenCode Harnesses

## Problem Statement
`ce-ai install` records every managed file with its SHA256 in `<harness-config>/compound-engineering/install-manifest.json`, and drift detection (`ce-ai status`, `ce-ai doctor`, `ce-ai workflow resume`) diffs on-disk files against that manifest.

A live-host audit found the guarantee holds only for OpenCode:
- `~/.config/opencode/compound-engineering/install-manifest.json`: ~395 files with real SHA256 digests.
- `~/.claude/compound-engineering/install-manifest.json`: `"files": []`.
- `~/.kimi-code/compound-engineering/install-manifest.json`: `"files": []`.

**Root cause (verified in source)**: `install` populates the manifest correctly for every harness, but `ce-ai sync` (`src/commands/sync.rs`, registration-spec arm) *rewrites* the manifest of every table-driven harness (claude, kimi, codex, copier, cursor, grok, agy, fx) with `files: vec![]` on every sync — silently discarding all digests. After any sync, drift detection for those harnesses has no desired state and is permanently blind.

Compounding this, every drift probe loads the manifest **only** from `ctx.opencode_config_dir`:
- `probe_manifest_drift_count` (`src/commands/workflow.rs:474-486`)
- the diff section of `src/commands/doctor.rs:66-83`
- the drift section of `src/commands/status.rs:94-118`

So even with correct manifests, claude/kimi drift would never be surfaced.

## In-Scope
1. **Root-Cause Fix in `sync`**: The registration-spec arm rewrites the manifest with the *actual* on-disk state of the harness's managed tree (harvested SHA256 entries) instead of an empty list, preserving the prior manifest's `installed_at` and `config_mutations`, and propagating write errors instead of silencing them.
2. **Harvest Helper**: `InstallManifest::harvest(managed_dir)` in `src/opencode/manifest.rs` — recursive walk, per-file SHA256, deterministic sort, excludes `install-manifest.json` itself.
3. **Multi-Harness Drift Detection**: Extend `probe_manifest_drift_count` (workflow), the doctor diff section, and the status drift section to also diff claude and kimi manifests (when present) against their managed trees. Harnesses with missing manifests degrade silently.
4. **Output Compatibility**: OpenCode output formats (`diff: {kind} {path}`, `drift: none`, `drift: unknown (no install manifest)`) remain byte-identical; non-OpenCode findings are prefixed with the harness name (`diff: claude modified …`, `drift: kimi: missing …`).
5. **Empirical Verification**: Unit tests for `harvest`, a sync-level regression test proving a pre-existing empty claude/kimi manifest gains real hashes, and doctor/workflow drift tests over tempdir fixtures.

## Out-of-Scope
1. **Copying Content During Sync for Registration Harnesses**: Sync intentionally does not re-copy the managed tree for table-driven harnesses (their delivery path is MCP + adoption); this change fixes manifest *fidelity*, not content refresh.
2. **Harnesses Beyond claude/kimi in Drift Probes**: Only claude and kimi are added to drift detection (the two harnesses named in the audit with live managed trees). Other registration harnesses can opt in later with a one-line addition.
3. **Manifest Schema Changes**: The `InstallManifest` schema is unchanged; only the population of `files` is corrected.
4. **Backfilling Existing Empty Manifests**: Existing installs heal automatically on the next `ce-ai sync`; no migration command is added.

## Risk Evaluation & Mitigation
- **Risk (Behavior Change in `sync`)**: Sync now writes real hashes where it wrote none.
  - *Mitigation*: The previous behavior was a data-loss bug (emptying a correct manifest); harvested hashes reflect exactly what is on disk, so drift semantics stay truthful. A regression test pins the new behavior.
- **Risk (Doctor/Status New Findings)**: Users with tampered claude/kimi trees now see findings that were previously invisible.
  - *Mitigation*: This is the intended audit outcome; opencode-only outputs remain unchanged, so existing consumers/tests do not regress.
- **Risk (Error Silencing Removal)**: The registration-arm manifest write previously ignored errors (`let _ =`); it now propagates.
  - *Mitigation*: Writes use the existing `write_atomic` path; failure to persist a manifest is a genuine sync failure and must surface (invariant #5, no dummy fallbacks).
