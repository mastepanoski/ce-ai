# Design: Cursor sessionStart Lifecycle Hook Integration

## Data Models & Schema

### 1. Cursor Hooks Configuration Schema
```json
{
  "version": 1,
  "hooks": {
    "sessionStart": [
      {
        "command": "ce-ai workflow resume --json"
      }
    ]
  }
}
```

### 2. Module Constants & Helpers (`src/harness/cursor.rs`)
```rust
pub const CURSOR_RESUME_COMMAND: &str = "ce-ai workflow resume --json";

pub fn has_session_start_hook(hooks_path: &Path) -> bool;
pub fn ensure_session_start_hook(hooks_path: &Path) -> Result<bool, CeError>;
pub fn remove_session_start_hook(hooks_path: &Path) -> Result<bool, CeError>;
```

### 3. Workflow JSON Schema Enhancement (`src/commands/workflow.rs`)
Add `"additional_context": additional_context` alongside `"additionalContext"` to serve both Cursor (snake_case) and Copilot (camelCase).

### 4. Integration Wiring
- **`src/commands/init_prj.rs`**: When `.cursor` exists, call `crate::harness::cursor::ensure_session_start_hook(&cursor_dir.join("hooks.json"))`.
- **`src/commands/deinit_prj.rs`**: Clean up `.cursor/rules/compound-engineering.mdc` and call `crate::harness::cursor::remove_session_start_hook(&cursor_dir.join("hooks.json"))`.
- **`src/commands/doctor.rs`**: Check `.cursor/hooks.json` using `has_session_start_hook` and report remediation finding if absent.
