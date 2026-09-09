# Technical Design: Kimi Code Native Plugin Manager Divergence

## System Architecture

The probe is an isolated, read-only diagnostic inside `ce-ai doctor`, delegating domain analysis to `src/harness/kimi.rs` — the same architecture as the Claude #327 probe.

```
                +--------------------------------+
                |      ce-ai doctor (CLI)        |
                +---------------+----------------
                                |
        check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir)
        check_kimi_orphan_managed_tree(&kimi_dir)
                                |
                                v
+-------------------------------------------------------------------------+
| src/harness/kimi.rs                                                     |
|                                                                         |
| 1. Find 'kimi' entry in state.installed_harnesses (scope vs cwd)        |
| 2. Resolve ce_version (harness entry -> release_provenance.tag)         |
| 3. Read <kimi_dir>/plugins/installed.json (tolerant, array-shaped)      |
| 4. Filter: id == 'compound-engineering*' && enabled                     |
| 5. Native version: <root>/package.json | plugin.json 'version'          |
| 6. Compare normalized versions -> Vec<KimiMarketplaceDivergence>        |
|                                                                         |
| Orphan probe:                                                           |
| 1. <kimi_dir>/compound-engineering/install-manifest.json exists?        |
| 2. Parse config.toml 'extra_skill_dirs'                                 |
| 3. Managed dir referenced? -> Option<KimiOrphanManagedTree>             |
+-------------------------------------------------------------------------+
                                |
                    Vec<Divergence> / Option<Orphan>
                                |
                                v
                +---------------+----------------+
                | 'doctor-info:' / 'doctor-warn:'|
                | (non-blocking, exit code 0)    |
                +--------------------------------+
```

## Data Schema & Structs

### 1. Deserialization Models (internal to `src/harness/kimi.rs`)
```rust
#[derive(Debug, Deserialize, Default)]
struct NativeInstalledJson {
    #[serde(default)]
    plugins: Vec<NativeInstalledPlugin>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct NativeInstalledPlugin {
    #[serde(default)]
    id: String,
    #[serde(default)]
    root: Option<PathBuf>,
    #[serde(default)]
    enabled: bool,
}
```

### 2. Domain Representation (`src/harness/kimi.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiMarketplaceDivergence {
    pub plugin_id: String,
    pub native_version: String,
    pub ce_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KimiOrphanManagedTree {
    pub managed_dir: PathBuf,
}
```

## API / Function Contracts

### `check_kimi_marketplace_divergence`
```rust
pub fn check_kimi_marketplace_divergence(
    state: &crate::state::state::State,
    cwd: &Path,
    kimi_dir: &Path,
) -> Vec<KimiMarketplaceDivergence>
```
1. **Harness Guard**: locate the `kimi` entry in `state.installed_harnesses`; `workspace` scope applies only when `target_dir` encompasses `cwd`; `global`/absent scope always applies. No entry ⇒ `Vec::new()`.
2. **Version Resolution**: `ce_version` = harness entry `version`, fallback `state.release_provenance.tag`. None ⇒ `Vec::new()`.
3. **File I/O**: read `<kimi_dir>/plugins/installed.json`; missing/unreadable/malformed ⇒ `Vec::new()`.
4. **Plugin Matching**: entries with `id == "compound-engineering"` or `id.starts_with("compound-engineering@")`, and `enabled == true`.
5. **Native Version**: read `<root>/package.json`, fallback `<root>/plugin.json`, extract `version` string; unresolvable ⇒ skip entry.
6. **Comparison**: `normalize_plugin_version(native) != normalize_plugin_version(ce)` ⇒ push divergence.

### `check_kimi_orphan_managed_tree`
```rust
pub fn check_kimi_orphan_managed_tree(kimi_dir: &Path) -> Option<KimiOrphanManagedTree>
```
1. Return `None` unless `<kimi_dir>/compound-engineering/install-manifest.json` exists.
2. Parse `<kimi_dir>/config.toml`; collect `extra_skill_dirs` string entries (missing/unparseable file ⇒ empty set).
3. The managed dir is "referenced" when any `extra_skill_dirs` entry equals it, or canonicalized equality succeeds.
4. Not referenced ⇒ `Some(KimiOrphanManagedTree { managed_dir })`.

### Shared Version Normalization
Reuse `crate::harness::claude::normalize_plugin_version` (no duplicate implementation).

## Doctor CLI Output Contract

In `src/commands/doctor.rs`, after the Claude marketplace probe:
```rust
let kimi_dir = HarnessKind::Kimi.harness_dir(&home_dir);
let kimi_divergences =
    crate::harness::kimi::check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
for d in &kimi_divergences {
    println!(
        "doctor-info: kimi native plugin divergence detected for '{}' : native plugin manager has v{} but ce-ai managed harness is v{} (update the plugin via Kimi's native plugin manager, or uninstall it to use the ce-ai managed tree exclusively)",
        d.plugin_id,
        crate::harness::claude::normalize_plugin_version(&d.native_version),
        crate::harness::claude::normalize_plugin_version(&d.ce_version),
    );
}
if let Some(orphan) = crate::harness::kimi::check_kimi_orphan_managed_tree(&kimi_dir) {
    println!(
        "doctor-warn: kimi managed tree '{}' is not referenced by Kimi config (extra_skill_dirs) — the ce-ai managed skills are inactive for Kimi (reference the tree from ~/.kimi-code/config.toml or remove it with 'ce-ai uninstall --harness kimi')",
        orphan.managed_dir.display()
    );
}
```

## Invariants & Compliance
1. **Strict Read-Only**: No write/mutation calls under `<kimi_dir>/plugins/`; `config.toml` is read, never written.
2. **Non-Fatal Reporting**: Divergences print `doctor-info:`, orphan trees print `doctor-warn:`; neither enters `findings`; exit code unchanged.
3. **Tolerant Deserialization**: All parsing uses `let Ok(..) = .. else` guards; malformed input degrades to no-finding.
4. **Atomic Writes**: N/A — this change performs no writes at all.
