# Technical Design: SHA256 Manifest Coverage for Non-OpenCode Harnesses

## System Architecture

```
ce-ai sync ──► registration-spec arm (per table-driven harness)
                     │
                     v
       InstallManifest::harvest(<harness>/compound-engineering/)
                     │  (walk + SHA256, sorted, minus install-manifest.json)
                     v
       InstallManifest { version, files: harvested, installed_at: prior,
                           config_mutations: prior } .write(config_dir)?
                     │
                     v
   <harness>/compound-engineering/install-manifest.json  (real digests)

ce-ai doctor / status / workflow resume
                     │
                     v
   for (harness, config_dir) in [opencode, claude, kimi]  (manifest present?)
       diff(manifest.files, manifest.files, config_dir/compound-engineering)
                     │
                     v
   findings / drift lines / manifest_drift_count
```

## Data Schema & Structs

No schema changes. `InstallManifest` and `ManifestFile` (`src/opencode/manifest.rs`) are unchanged.

### New Helper (`src/opencode/manifest.rs`)
```rust
impl InstallManifest {
    /// Harvests SHA256 entries for every file currently under the managed
    /// dir (excluding install-manifest.json itself), sorted by path.
    pub fn harvest(managed_dir: &Path) -> Vec<ManifestFile>
}
```
Implementation: recursive `std::fs` walk (files only), `crate::state::diff::sha256_hex` per file, paths relative to `managed_dir` with `/` separators, `install-manifest.json` excluded, `BTreeMap`-ordered output.

### Sync Fix (`src/commands/sync.rs`, registration-spec arm)
```rust
} else if let Some(spec) = registration_spec(h_kind) {
    spec.register_companions(&target_config)?;
    let managed_dir = config_dir.join(MANAGED_DIR);
    let prior = InstallManifest::load(&config_dir).ok();
    let manifest_path = managed_dir.join("install-manifest.json");
    if manifest_path.exists() {
        arm!(&manifest_path);
        InstallManifest {
            version: version.to_string(),
            plugin_name: "compound-engineering".into(),
            installed_at: prior.as_ref().map(|m| m.installed_at.clone())
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
            source: source_json.clone(),
            files: InstallManifest::harvest(&managed_dir),
            config_mutations: prior.map(|m| m.config_mutations).unwrap_or_default(),
        }
        .write(&config_dir)?;   // errors propagate (no `let _ =`)
    }
}
```

## Multi-Harness Drift Detection

Shared harness list (new helper in `src/commands/workflow.rs` or `src/opencode/manifest.rs`):
```rust
/// Harnesses whose managed-tree manifests participate in drift detection
/// beyond OpenCode itself.
pub const DRIFT_PROBE_HARNESSES: [HarnessKind; 2] =
    [HarnessKind::Claude, HarnessKind::Kimi];
```

### `probe_manifest_drift_count(ctx)` (workflow.rs)
1. Existing OpenCode logic unchanged (early-return semantics preserved).
2. Then: `let home = crate::harness::home_dir_from_ctx(ctx);` for each of `[Claude, Kimi]`: `config_dir = kind.harness_dir(&home)`; if `InstallManifest::load(&config_dir)` is Ok and non-empty, diff against `config_dir/MANAGED_DIR` and add `actions.len()` to the total.

### Doctor diff section (doctor.rs)
1. Keep the existing OpenCode block byte-identical in output (`diff: {kind} {path}`).
2. After it, loop `[Claude, Kimi]`: load manifest; on success diff and push `diff: {harness} {kind} {path}` per action. Missing manifest ⇒ skip silently.

### Status drift section (status.rs)
Restructure into an iterator over `(Option<harness_label>, config_dir)` = opencode + claude + kimi:
- opencode actions print `drift: {kind} {path}` (unchanged);
- claude/kimi actions print `drift: {harness}: {kind} {path}`;
- if no manifest loaded at all ⇒ `drift: unknown (no install manifest)` (unchanged);
- else if zero total actions ⇒ `drift: none` (unchanged).

## API / Function Contracts
- `InstallManifest::harvest(managed_dir: &Path) -> Vec<ManifestFile>` — pure read; never writes; returns empty vec for missing dirs.
- `probe_manifest_drift_count(ctx: &Context) -> usize` — same signature; widened coverage.

## Invariants & Compliance
1. **Atomic Writes**: Manifest writes go through `InstallManifest::write` → `crate::state::write_atomic`. No raw `std::fs::write`.
2. **No Silenced Errors**: The registration-arm manifest write propagates errors with `?`.
3. **User Config Preservation**: `sync` continues to only register companions into harness configs via the existing per-harness merge functions; this change adds no new config writes.
4. **Read-Only Probes**: Drift detection never mutates; it only hashes and compares.
