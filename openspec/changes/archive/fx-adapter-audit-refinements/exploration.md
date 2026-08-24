# Exploration: Vercel Labs fx Adapter Audit Refinements

## Audit Analysis

### 1. Filesystem-Dependent Path Resolution (`home.join("mcp.json").exists()`)
- **Status**: `fx.rs:25-27` checked if `home.join("mcp.json").exists()`. If `$HOME/mcp.json` pre-existed, passing `$HOME` to `default_config_path` returned `$HOME/mcp.json` instead of `$HOME/.fx/mcp.json`.
- **Action**: Remove `.exists()` check. Rely solely on basename matching (`home.file_name() == Some("mcp.json")` or `Some(".fx")`).

### 2. Error Silencing on File Removal
- **Status**: `fx.rs:134` silenced `remove_file` errors with `let _ = std::fs::remove_file(...)`.
- **Action**: Ignore `io::ErrorKind::NotFound` while returning `CeError::Io` on real filesystem errors.

### 3. `FX_HOME` Environment Variable Extension
- **Status**: Official `fx` documentation lists `FX_*` variables for runtime execution, but `FX_HOME` is a `ce-ai` extension for custom directory relocation.
- **Action**: Formally document `FX_HOME` as a `ce-ai` extension in `design.md`.

### 4. Managed Server Collision Handling
- **Status**: `fx.rs:95-96` removes `type` from `extra` map (`existing_extra.remove("type")`) before inserting updated entry.
- **Action**: Formally document this pattern in `design.md`.
