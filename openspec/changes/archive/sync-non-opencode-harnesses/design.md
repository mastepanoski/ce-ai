# Technical Design: Harness-Neutral Sync & Upgrade

## Architectural Changes

### 1. Source & Version Resolution (`resolve_sync_source_and_version`)
In `src/commands/sync.rs`, add a helper function:
```rust
fn resolve_sync_source(
    ctx: &Context,
    state: &State,
    home_dir: &Path,
    opencode_dir: &Path,
) -> Result<(PathBuf, String, serde_json::Value), CeError>
```
Resolution order:
1. If `InstallManifest::load(opencode_dir)` is Ok: use `(resolve_source_root(&manifest.source)?, manifest.version, manifest.source)`.
2. Iterate `state.installed_harnesses`:
   - Attempt `InstallManifest::load` on the harness target directory (custom `plugins_dir` or native harness dir). If Ok, resolve source.
   - If manifest file is missing but the entry contains `"source"` and `"version"`: resolve source.
3. Check `state.release_provenance`: construct `{ "kind": "github-release", "tag": prov.tag, "tree": prov.extraction_path }`.
4. If no source could be resolved:
   - If `state.installed_harnesses.is_empty()`: return `Err(CeError::Runtime("no harnesses installed — run ce-ai install first".into()))`.
   - Else: return `Err(CeError::Runtime("no install-manifest.json — run install first".into()))`.

### 2. Gated OpenCode Sync in `sync_with`
Move active harnesses detection to the top of `sync_with`:
```rust
let mut active_harnesses: Vec<String> = state
    .installed_harnesses
    .iter()
    .filter_map(|h| h["name"].as_str().map(|s| s.to_string()))
    .collect();
for h in HarnessKind::detect_ce_installed_harnesses(&home_dir) {
    let name = h.to_string();
    if !active_harnesses.contains(&name) {
        active_harnesses.push(name);
    }
}
let opencode_manifest = InstallManifest::load(&opencode_dir).ok();
let opencode_active = active_harnesses.iter().any(|h| h == "opencode") || opencode_manifest.is_some();
```
When `opencode_active`:
- Compute OpenCode diff against `desired` and execute file copies/restores/removals.
- Write updated `InstallManifest` to `&opencode_dir`.
- Sync model assignments and check drift for the matrix.
- Push `opencode` to `surfaces`.

When `!opencode_active`:
- Skip OpenCode directory diff, mutations, and manifest write.
- Skip model assignments import/purge if `opencode.json` does not exist.
- Do NOT push `opencode` to `surfaces`.

### 3. Native & Custom Harness Sync in `sync_with`
- Continue processing all `active_harnesses` in the loop:
  - `Custom`: copies plugins and skills, writes `InstallManifest` under `cfg.plugins_dir`, and verifies against desired tree.
  - Table-driven harnesses (`Claude`, `Cursor`, `Copilot`, etc.): invokes `spec.register_companions(&target_config)`.
  - Also update `install-manifest.json` under `config_dir` if present.
- Adopted skill surfaces are checked and drift restored.
- Verification matrix displays rows for all active harnesses.
