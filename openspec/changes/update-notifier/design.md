# Design: Automated Background Update Notifier for ce-ai CLI & Harness Releases

## Architecture & Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                       ce-ai main                        │
│                                                         │
│ 1. Check suppression (CI, env, quiet, is_terminal,      │
│    state.json config)                                   │
│ 2. Read ~/.ce-ai/cache/update_check.json                │
│ 3. If stale (>24h) and not suppressed:                  │
│    Spawn detached background thread                     │
│    │                                                    │
│    ▼ [Background Thread]                                │
│    - GET https://api.github.com/.../releases/latest     │
│    - Timeout: 3s, fail-closed                           │
│    - write_atomic(~/.ce-ai/cache/update_check.json)     │
│                                                         │
│ 4. Main Thread runs subcommand immediately (0ms delay)  │
│                                                         │
│ 5. On Subcommand Completion:                            │
│    - If latest_version > current_version:               │
│      Print banner to stderr                             │
│ 6. exit(code)                                           │
└─────────────────────────────────────────────────────────┘
```

## Data Schemas

### 1. Cache Schema (`<config_dir>/cache/update_check.json`)
```json
{
  "last_checked_at": "2026-09-26T01:00:00Z",
  "latest_version": "1.70.0",
  "latest_tag": "v1.70.0",
  "release_url": "https://github.com/mastepanoski/ce-ai/releases/tag/v1.70.0"
}
```

Rust struct:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCheckCache {
    pub last_checked_at: String,
    pub latest_version: String,
    pub latest_tag: String,
    pub release_url: String,
}
```

### 2. State Configuration Schema (`state.json`)
```json
{
  "update_notifier": {
    "enabled": true,
    "interval_hours": 24
  }
}
```

Rust struct:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateNotifierConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_update_check_interval_hours")]
    pub interval_hours: u64,
}

fn default_update_check_interval_hours() -> u64 {
    24
}
```

## Module Structure: `src/source/update_notifier.rs`

### Public API
- `pub fn should_check_updates(ctx: &Context, state: &State) -> bool`: Evaluates `--quiet`, `CE_NO_UPDATE_NOTIFIER`, `CI`, `GITHUB_ACTIONS`, `is_terminal()`, and `state.update_notifier.enabled`.
- `pub fn read_cache(config_dir: &Path) -> Option<UpdateCheckCache>`: Reads and parses `<config_dir>/cache/update_check.json`.
- `pub fn is_cache_stale(cache: &UpdateCheckCache, interval_hours: u64) -> bool`: Checks if `last_checked_at` is older than `interval_hours` (default 24h).
- `pub fn spawn_background_check(config_dir: PathBuf, token: Option<String>)`: Spawns a background thread that executes the HTTP release fetch and writes the cache via `write_atomic`.
- `pub fn check_and_update_cache_sync(config_dir: &Path, client: &reqwest::blocking::Client, token: Option<&str>) -> Result<UpdateCheckCache, CeError>`: Synchronous check used by `self_update` and testing.
- `pub fn format_update_banner(current_version: &str, latest_version: &str, release_url: &str) -> String`: Generates the formatted Unicode box banner.
- `pub fn maybe_print_update_notification(ctx: &Context, state: &State)`: Called at CLI exit; checks cache against current version and writes to `stderr`.

## Banner Formatting
```
╭─────────────────────────────────────────────────────────────────╮
│  Update available: ce-ai v1.69.1 → v1.70.0                      │
│  Run 'ce-ai self-update' or 'brew upgrade ce-ai' to upgrade     │
╰─────────────────────────────────────────────────────────────────╯
```

## Integration Points
1. `src/main.rs`:
   - Initialize background check if stale and not suppressed before dispatch.
   - Print notification banner to `stderr` after command execution.
2. `src/commands/self_update.rs`:
   - Refresh `<config_dir>/cache/update_check.json` whenever `ce-ai self-update --check` or upgrade runs.
3. `src/commands/doctor.rs`:
   - Inspect cache freshness and configuration; emit `doctor-info: update-notifier: ...`.
