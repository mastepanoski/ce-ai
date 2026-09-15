---
title: ce-ai sync wiped SHA256 install-manifest files to empty for non-OpenCode harnesses
date: 2026-09-09
category: logic-errors
module: commands/sync, opencode/manifest, commands/workflow, commands/doctor, commands/status
problem_type: logic_error
component: tooling
symptoms:
  - "~/.claude/compound-engineering/install-manifest.json and ~/.kimi-code/compound-engineering/install-manifest.json contained \"files\": [] while the opencode manifest held ~395 SHA256 entries"
  - "ce-ai doctor, ce-ai status, and ce-ai workflow resume reported clean drift for claude/kimi even when the managed tree was corrupted or stale"
  - "any ce-ai sync run silently destroyed hashes that ce-ai install had correctly populated"
root_cause: logic_error
resolution_type: code_fix
severity: high
tags: [sync, install-manifest, sha256, drift-detection, registration-harness, atomic-writes]
applies_when: "When ce-ai sync rewrites install-manifest.json for every table-driven registration harness (claude, kimi, codex, copilot, cursor, grok, agy, fx)."
---

# ce-ai sync wiped SHA256 install-manifest files to empty for non-OpenCode harnesses

## Problem

`ce-ai sync` rewrites `install-manifest.json` for every table-driven registration harness (claude, kimi, codex, copilot, cursor, grok, agy, fx). The registration-spec arm of the sync loop (src/commands/sync.rs) built the replacement manifest with `files: vec![]`, so each sync destroyed the per-file SHA256 index that `ce-ai install` had correctly populated — and, worse, the write was fire-and-forget (`let _ = ... .write(...)`), so a failed write also went unreported. Because claude/kimi drift was probed only through the opencode manifest, `doctor`/`status`/`workflow resume` all reported "clean" while the managed trees for those harnesses were corrupt or outdated.

## Symptoms

- `~/.claude/compound-engineering/install-manifest.json` and `~/.kimi-code/compound-engineering/install-manifest.json` showed `"files": []` while `~/.config/opencode/compound-engineering/install-manifest.json` had ~395 SHA256 entries.
- `ce-ai doctor`, `ce-ai status`, and `ce-ai workflow resume` reported no drift for claude/kimi even after tampering with or deleting files in those managed trees.
- A fresh `ce-ai install` restored correct hashes; the very next `ce-ai sync` wiped them again.
- Related CI failure on windows-latest: `test_kimi_orphan_managed_tree_matrix` failed only on Windows while passing on linux/macOS.

## What Didn't Work

- Re-running `ce-ai install`: it repopulated the manifests, but any subsequent `ce-ai sync` re-wiped them — the bug was in sync's registration-spec arm, not in install.
- Trusting `doctor`/`status` "no drift" output as ground truth: drift probing only consulted the opencode manifest, so the corrupted claude/kimi manifests were invisible to every health surface. Empty `files: []` is indistinguishable from "nothing to check" when nothing reads the other manifests.
- Formatting TOML by hand in tests (`format!("extra_skill_dirs = [\"{}\"]", managed_dir.display())`): on Windows the backslashes in the path are invalid TOML escapes (`\U...`), so `config.toml` failed to parse, the orphan-tree check degraded to "not referenced", and the assertion failed only on windows-latest. String-formatted paths are not a substitute for a TOML serializer or normalized path comparison.

## Solution

Merged in PRs #335/#336 (v1.48.0), green across the full CI matrix (linux/mac/windows) plus the Docker E2E gate.

### 1. New `InstallManifest::harvest` (src/opencode/manifest.rs)

Collects `{path, sha256}` for every file present on disk under the managed dir (excluding `install-manifest.json` itself), sorted deterministically, never writes.

### 2. sync registration arm rewritten to harvest + preserve + propagate (src/commands/sync.rs)

Before (destroyed hashes, swallowed errors, dropped metadata):

```rust
spec.register_companions(&target_config)?;
let manifest_path = config_dir.join(MANAGED_DIR).join("install-manifest.json");
if manifest_path.exists() {
    arm!(&manifest_path);
    let _ = InstallManifest {
        version: version.to_string(),
        plugin_name: "compound-engineering".into(),
        installed_at: Utc::now().to_rfc3339(),
        source: source_json.clone(),
        files: vec![],
        config_mutations: vec![],
    }
    .write(&config_dir);
}
```

After (harvested hashes, prior metadata preserved, errors propagate with `?`, and no manifest is fabricated where none existed):

```rust
spec.register_companions(&target_config)?;
let manifest_path = config_dir.join(MANAGED_DIR).join("install-manifest.json");
if manifest_path.exists() {
    let prior = InstallManifest::load(&config_dir).ok();
    arm!(&manifest_path);
    InstallManifest {
        version: version.to_string(),
        plugin_name: "compound-engineering".into(),
        installed_at: prior
            .as_ref()
            .map(|m| m.installed_at.clone())
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
        source: source_json.clone(),
        files: InstallManifest::harvest(&config_dir.join(MANAGED_DIR)),
        config_mutations: prior.map(|m| m.config_mutations).unwrap_or_default(),
    }
    .write(&config_dir)?;
}
```

### 3. Multi-harness drift probing

`probe_manifest_drift_count` (src/commands/workflow.rs) now sums drift across opencode plus the registration harnesses via `DRIFT_PROBE_HARNESSES`, and the doctor diff probe does the same. `status` (src/commands/status.rs) reports a per-harness drift section with explicit contracts — `drift: none` when every manifest matches its on-disk tree, `drift: unknown (no install manifest)` when no manifest exists at all, and `drift: <harness>: <kind> <path>` lines otherwise.

### 4. Windows CI test fix

`test_kimi_orphan_managed_tree_matrix` now serializes `config.toml` with the `toml` crate instead of `format!` with raw paths, and compares canonicalized paths rather than display strings, so Windows backslashes can no longer produce invalid TOML escapes or mismatched string forms.

## Why This Works

The root cause was a semantic lie in the manifest: the registration arm rewrote the manifest from the *sync plan's* perspective (it only tracks opencode file copies), and since registration harnesses deliver skills via adoption rather than file copies, the arm "had" no files — so it wrote an empty list. But a manifest's job is to describe what *should* be on disk and hash it, not to mirror one command's internal plan. Harvesting hashes directly from the managed tree restores that meaning: the manifest always reflects reality at sync time, so the diff engine (desired vs. on-disk) has a truthful baseline for every harness. Preserving `installed_at`/`config_mutations` keeps provenance and mutation history intact instead of fabricating fresh timestamps. Propagating the write error with `?` converts silent corruption into a loud `CeError` (exit code per the invariant table). Extending drift probing to claude/kimi closes the observability hole — the health surfaces can no longer certify a harness whose manifest they never read. Gating the rewrite behind `manifest_path.exists()` means sync never invents manifests for harnesses that were never installed.

## Prevention

- Regression test `sync_with_harvests_real_hashes_for_registration_harness_manifests` (src/commands/tests/sync.rs) locks in the invariant: after `sync_with`, registration-harness manifests must contain the real harvested hashes, never `files: []`.
- Hard rule: never `let _ =` on a manifest write (or any state-mutating I/O). Writes go through `crate::state::write_atomic` and errors propagate with `?` — invariant #5 (no dummy fallbacks) applies to sync just as it does everywhere else.
- Treat a registration-harness manifest with `files: []` after a sync as an alarm signal: for these harnesses the managed tree is never legitimately empty, so an empty file list means harvest ran against a missing tree or the wipe bug regressed. Investigate before shipping.
- Serialize TOML with the `toml` crate in tests and production code, never with `format!` embedding `Path::display()` — raw Windows paths inject invalid `\U`-style escapes.
- Compare paths canonicalized/normalized, never as display strings; `Path::display()` output is platform-dependent and must not cross an assertion boundary.
- When adding a new health/drift surface, enumerate *all* relevant manifests up front (as `probe_manifest_drift_count` now does with `DRIFT_PROBE_HARNESSES`) instead of defaulting to the opencode manifest.

## Related Issues

- PR #335 — fix: restore SHA256 manifest coverage for non-OpenCode harnesses
- PR #336 — feat(doctor): detect Kimi native plugin marketplace divergence and orphan managed tree
- Related learnings: `multi-harness-support-implementation.md` and `multi-harness-propagation-and-sync-verification.md` document the sync machinery this fix repaired (refreshed in v1.48.0 via ce-compound-refresh — their verification-matrix claims now state that per-harness hash integrity became real for non-OpenCode harnesses only in v1.48.0)
- src/opencode/manifest.rs (`InstallManifest::harvest`), src/commands/sync.rs (registration arm), src/commands/workflow.rs (`probe_manifest_drift_count`), src/commands/status.rs (multi-harness drift section)
