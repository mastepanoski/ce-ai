# Design: Workspace-Scoped OpenCode Manifest Resolution

## Data Model & Schema Changes

### `state.installed_harnesses` Entry Format
Additive fields added to the `serde_json::Value` objects inside `state.installed_harnesses`:
```json
{
  "name": "opencode",
  "version": "1.37.1",
  "source": { "kind": "local", "path": "..." },
  "installed_at": "2026-09-04T20:00:00Z",
  "last_synced_at": "2026-09-04T20:00:00Z",
  "scope": "workspace",
  "target_dir": "/path/to/repo"
}
```
For global installs:
```json
{
  "name": "opencode",
  "version": "1.37.1",
  "source": { "kind": "github-release", "tag": "v1.37.1" },
  "installed_at": "2026-09-04T20:00:00Z",
  "last_synced_at": "2026-09-04T20:00:00Z",
  "scope": "global"
}
```

## Context Helper Function

Add `resolve_opencode_dir(&self, state: &State) -> PathBuf` to `Context` (`src/commands/mod.rs`):
```rust
impl Context {
    /// Resolves the effective OpenCode configuration directory.
    ///
    /// If executed within a workspace where `state.installed_harnesses` contains a
    /// workspace-scoped "opencode" entry for this workspace, or if the current
    /// workspace contains `<workspace>/compound-engineering/install-manifest.json`,
    /// returns the workspace directory. Otherwise returns `self.opencode_config_dir`.
    pub fn resolve_opencode_dir(&self, state: &State) -> PathBuf {
        if let Some(ws) = &self.workspace_root {
            // 1. Check if state has an entry explicitly targeting this workspace
            let has_ws_entry = state.installed_harnesses.iter().any(|h| {
                h["name"].as_str() == Some("opencode")
                    && h["scope"].as_str() == Some("workspace")
                    && h["target_dir"].as_str().map(Path::new) == Some(ws.as_path())
            });
            if has_ws_entry {
                return ws.clone();
            }

            // 2. Check if a workspace manifest exists on disk
            let ws_manifest = ws.join(crate::opencode::plugins::MANAGED_DIR).join("install-manifest.json");
            if ws_manifest.exists() {
                return ws.clone();
            }
        }

        self.opencode_config_dir.clone()
    }
}
```

## Subcommand Adaptations

### 1. `src/commands/install.rs`
- In `run`: when pushing an entry to `state.installed_harnesses`, add `"scope": scope_arg`. If `scope_arg == "workspace"`, add `"target_dir": config_dir.display().to_string()`.

### 2. `src/commands/doctor.rs`
- Load `state` earlier or use `ctx.resolve_opencode_dir(&state)`.
- Use `opencode_dir` for:
  - `opencode.json` (`opencode_dir.join("opencode.json")`)
  - `InstallManifest::load(&opencode_dir)`
  - `diff::diff(..., &opencode_dir.join(MANAGED_DIR))`
  - `crate::opencode::plugins::has_session_start_plugin(&opencode_dir)`
  - `read_config(&opencode_json)` for model drift

### 3. `src/commands/status.rs`
- Use `ctx.resolve_opencode_dir(&state)` for loading manifest and checking drift.

### 4. `src/commands/sync.rs`
- Use `ctx.resolve_opencode_dir(&state)` for loading manifest, managed dir, and rewriting manifest.
