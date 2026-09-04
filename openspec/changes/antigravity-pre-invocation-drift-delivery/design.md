# Design: Antigravity PreInvocation Turn-0 Drift Delivery

## Data Structures & CLI Contract

### 1. `ce-ai workflow resume` Extension
```rust
pub enum Action {
    ...
    Resume {
        #[arg(long)]
        json: bool,
        /// Antigravity PreInvocation hook mode (reads stdin, dedupes per conversationId, injects ephemeralMessage).
        #[arg(long)]
        pre_invocation: bool,
    },
}
```

### 2. Antigravity Hook Helpers (`src/harness/agy.rs`)
```rust
pub const AGY_RESUME_COMMAND: &str = "ce-ai workflow resume --pre-invocation";

pub fn has_pre_invocation_hook(hooks_path: &Path) -> bool;
pub fn ensure_pre_invocation_hook(hooks_path: &Path) -> Result<bool, CeError>;
pub fn remove_pre_invocation_hook(hooks_path: &Path) -> Result<bool, CeError>;

// Backward-compatible alias helpers
pub use has_pre_invocation_hook as has_session_start_hook;
pub use ensure_pre_invocation_hook as ensure_session_start_hook;
pub use remove_pre_invocation_hook as remove_session_start_hook;
```

### 3. Hook Payload Schema (`hooks.json`)
```json
{
  "compound-engineering": {
    "PreInvocation": [
      {
        "type": "command",
        "command": "ce-ai workflow resume --pre-invocation"
      }
    ]
  }
}
```

### 4. Integration Wiring
- **`src/commands/init_prj.rs`**: When `.agents` exists, call `ensure_pre_invocation_hook(&agents_dir.join("hooks.json"))`.
- **`src/commands/deinit_prj.rs`**: Call `remove_pre_invocation_hook(&agents_dir.join("hooks.json"))` and prune `.agents` directory if empty.
- **`src/commands/doctor.rs`**: Check `.agents/hooks.json` using `has_pre_invocation_hook` and surface remediation hint if absent.
