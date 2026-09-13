# Exploration: SHA256 Manifest Coverage for Non-OpenCode Harnesses

## Technical Investigation

### Reproduction (live host, 2026-09-08)
```bash
$ python3 -c "import json;print(len(json.load(open('/Users/mastepanoski/.config/opencode/compound-engineering/install-manifest.json'))['files']))"
395
$ python3 -c "import json;print(json.load(open('/Users/mastepanoski/.claude/compound-engineering/install-manifest.json'))['files'])"
[]
$ python3 -c "import json;print(json.load(open('/Users/mastepanoski/.kimi-code/compound-engineering/install-manifest.json'))['files'])"
[]
```
Both empty manifests carry a recent `installed_at` and `version: compound-engineering-v3.24.0`, matching a sync run (sync rewrites `installed_at` with `Utc::now()`; install wrote the original populated manifest).

### Root Cause
`src/commands/install.rs` populates `files` correctly for **all** non-custom harnesses: the managed-file copy loop (install.rs:268-299) pushes one `ManifestFile { path, sha256 }` per copied file *before* the per-kind manifest write branches.

`src/commands/sync.rs` then destroys that data. In `sync_with`, the registration-spec arm (sync.rs:458-476) executes once per table-driven harness (claude, kimi, codex, copilot, cursor, grok, agy, fx):

```rust
let manifest_path = config_dir.join(MANAGED_DIR).join("install-manifest.json");
if manifest_path.exists() {
    arm!(&manifest_path);
    let _ = InstallManifest {
        version: version.to_string(),
        ...
        files: vec![],          // <-- data loss: digest list wiped on every sync
        config_mutations: vec![],
    }
    .write(&config_dir);
}
```

Two additional defects in the same block:
1. `let _ =` silences write errors (violates invariant #5, no dummy fallbacks).
2. `installed_at` and `config_mutations` from the prior manifest are discarded.

Note: sync deliberately does **not** re-copy content for registration harnesses (their delivery path is MCP registration + skill adoption; see the R4 comment in both install.rs and sync.rs). Therefore the source of truth available at sync time for these harnesses is the **on-disk managed tree itself**, not the release source — the correct fix is to harvest hashes from disk.

### Second Layer: Single-Harness Drift Probes
Even with correct manifests, no probe would read them:
- `probe_manifest_drift_count(ctx)` (workflow.rs:474-486) loads `InstallManifest::load(&ctx.opencode_config_dir)` only.
- `doctor.rs:66-83` diffs only `opencode_dir.join(MANAGED_DIR)`.
- `status.rs:94-118` prints drift only for the OpenCode manifest.

### Existing Diff Engine
`crate::state::diff::diff(desired, manifest, fs_root)` plans `Copy`/`Restore`/`Remove` by hashing files under `fs_root`. It is harness-agnostic — only the manifest location and managed-dir root differ per harness. `HarnessKind::harness_dir(&home_dir)` already resolves claude (`~/.claude`, `CLAUDE_CONFIG_DIR` aware) and kimi (`~/.kimi-code`, `KIMI_CODE_HOME` aware) directories, and `home_dir_from_ctx(ctx)` resolves the home from the context — the same resolution the rtk and marketplace probes already use.

## Evaluated Options & Architectural Tradeoffs

| Option | Description | Pros | Cons | Verdict |
|---|---|---|---|---|
| **A: Sync re-copies managed trees for registration harnesses** | Make sync mirror install's copy loop for claude/kimi | Manifests could reuse `desired` hashes | Changes the R4 delivery contract (sync intentionally does not touch those trees); larger blast radius | Rejected |
| **B: Harvest hashes from the on-disk managed tree** | Walk `<harness>/compound-engineering/`, hash every file except `install-manifest.json` | Truthful (reflects exactly what is on disk); no delivery-contract change; fixes doctor/status/workflow simultaneously | Hashes are computed at sync time (I/O cost ~400 files, negligible) | **Selected** |
| **C: Keep manifests empty; drop manifest writes in sync** | Stop rewriting non-opencode manifests | Simplest | Leaves drift detection permanently blind for non-OpenCode harnesses — the audit's core complaint | Rejected |
| **D: Extend drift probes to all registration harnesses at once** | Add codex/cursor/grok/… to doctor/status/workflow loops | Uniform | Audited reality only evidences claude/kimi managed trees; wider blast radius without evidence | Rejected (claude+kimi only; extension is a one-line addition per harness) |

## Output Compatibility Analysis
- `status` prints `drift: none` when a manifest exists with zero actions, and `drift: unknown (no install manifest)` when OpenCode's manifest is missing (pinned by `tests/cli.rs:5604` and `status_prints_installed_harness_version_and_drift`). The multi-harness iteration must keep both strings byte-identical for the OpenCode-only cases.
- `doctor` findings use `diff: {kind} {path}`; new per-harness findings use `diff: {harness} {kind} {path}` to remain greppable and unambiguous.
- `workflow resume`'s `manifest_drift_count` is a count; summing across harnesses preserves the JSON schema while widening coverage.
