# Technical Design: Claude Code Native Marketplace Plugin Divergence

## System Architecture

The divergence probe operates as an isolated, read-only diagnostic within `ce-ai doctor`, delegating domain analysis to `src/harness/claude.rs`.

```
                  +--------------------------------+
                  |      ce-ai doctor (CLI)        |
                  +---------------+----------------+
                                  |
               check_claude_marketplace_divergence(&state, &cwd, &claude_dir)
                                  |
                                  v
+-------------------------------------------------------------------------+
| src/harness/claude.rs                                                  |
|                                                                         |
| 1. Find 'claude' entry in state.installed_harnesses                     |
| 2. Read <claude_dir>/plugins/installed_plugins.json (best-effort)      |
| 3. Filter keys starting with 'compound-engineering'                     |
| 4. Filter applicable entries (scope == 'user' || cwd within projectPath)|
| 5. Compare normalized versions (ce_version vs native_version)           |
| 6. Return Vec<ClaudeMarketplaceDivergence>                             |
+-------------------------------------------------------------------------+
                                  |
                        Vec<Divergence>
                                  |
                                  v
                  +---------------+----------------+
                  | Emit 'doctor-info:' per scope  |
                  | (non-blocking, exit code 0)    |
                  +--------------------------------+
```

## Data Schema & Structs

### 1. Deserialization Models (Internal to `src/harness/claude.rs`)
```rust
#[derive(Debug, Deserialize, Default)]
struct NativeInstalledPlugins {
    #[serde(default)]
    version: u32,
    #[serde(default)]
    plugins: BTreeMap<String, Vec<NativePluginEntry>>,
}

#[derive(Debug, Deserialize, Clone)]
struct NativePluginEntry {
    #[serde(default)]
    scope: String,
    #[serde(rename = "projectPath", default)]
    project_path: Option<PathBuf>,
    #[serde(rename = "installPath", default)]
    install_path: Option<PathBuf>,
    #[serde(default)]
    version: String,
}
```

### 2. Domain Representation (`src/harness/claude.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeMarketplaceDivergence {
    pub plugin_id: String,
    pub scope: String,
    pub native_version: String,
    pub ce_version: String,
    pub project_path: Option<PathBuf>,
}
```

## API / Function Contracts

### `normalize_plugin_version`
```rust
pub fn normalize_plugin_version(raw: &str) -> &str {
    let s = raw.trim();
    let s = s.strip_prefix("compound-engineering-").unwrap_or(s);
    let s = s.strip_prefix("compound-engineering@").unwrap_or(s);
    s.strip_prefix('v').unwrap_or(s)
}
```

### `check_claude_marketplace_divergence`
```rust
pub fn check_claude_marketplace_divergence(
    state: &State,
    cwd: &Path,
    claude_dir: &Path,
) -> Vec<ClaudeMarketplaceDivergence>
```
1. **Harness Guard**: Check if `claude` is installed in `state.installed_harnesses`. If not present or not applicable to `cwd`, return `Vec::new()`.
2. **Version Resolution**: Resolve `ce_version` from the harness entry (or fallback to `state.release_provenance.tag`). If no version is recorded, return `Vec::new()`.
3. **File I/O**: Attempt to read `<claude_dir>/plugins/installed_plugins.json`. If missing or unparseable, return `Vec::new()`.
4. **Plugin Matching**: Iterate over `plugins` map where key is `"compound-engineering"` or starts with `"compound-engineering@"`.
5. **Applicability & Comparison**:
   - For each entry:
     - `is_applicable`:
       - `entry.scope == "user"` -> true
       - `entry.scope == "project" || entry.scope == "local"`: true if `entry.project_path` is Some(`p`) and (`cwd == p || cwd.starts_with(p) || p.starts_with(cwd)`).
     - If applicable and `normalize_plugin_version(&entry.version) != normalize_plugin_version(&ce_version)`:
       - Push `ClaudeMarketplaceDivergence`.

## Doctor CLI Output Contract

In `src/commands/doctor.rs`:
```rust
let home_dir = crate::harness::home_dir_from_ctx(ctx);
let claude_dir = HarnessKind::Claude.harness_dir(&home_dir);
let repo_root = ctx.repo_root();
let cwd = std::env::current_dir().unwrap_or_else(|_| repo_root.clone());

let divergences = crate::harness::claude::check_claude_marketplace_divergence(
    &state,
    &cwd,
    &claude_dir,
);

for d in divergences {
    let marketplace = d
        .plugin_id
        .split_once('@')
        .map(|(_, m)| m)
        .unwrap_or("compound-engineering-plugin");
    let update_cmd = format!(
        "claude plugin marketplace update {marketplace} && claude plugin update {}",
        d.plugin_id
    );
    println!(
        "doctor-info: claude native plugin marketplace divergence detected for '{}' (scope: {}): native marketplace has v{} but ce-ai managed harness is v{} (run '{}' to update)",
        d.plugin_id, d.scope, d.native_version, d.ce_version, update_cmd
    );
}
```

## Invariants & Compliance
1. **Strict Read-Only**: Absolutely no write or mutation calls on `<claude_dir>/plugins/`.
2. **Non-Fatal Reporting**: Divergences are logged via `println!("doctor-info: ...")` and are never added to `findings`. Exit code remains `0` unless other findings exist.
3. **Tolerant Deserialization**: All JSON parsing uses safe `let Ok(val) = ...` or returns empty. Malformed files never trigger panics or runtime errors.
