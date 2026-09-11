# Technical Design: Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI

## 1. System Architecture

```
┌────────────────────────────────────────────────────────┐
│               GitHub Copilot CLI Session               │
└──────────────────────────┬─────────────────────────────┘
                           │ 1. Evaluates .github/hooks/hooks.json
                           │    and triggers sessionStart hook
                           ▼
┌────────────────────────────────────────────────────────┐
│            ce-ai workflow resume --json                │
│      (Generates formatted RepoState & JSON)            │
└──────────────────────────┬─────────────────────────────┘
                           │ 2. Returns stdout with:
                           │    { "additionalContext": "..." }
                           ▼
┌────────────────────────────────────────────────────────┐
│               Copilot Context Injection                │
│        (Injected into agent context at Turn 0)         │
└────────────────────────────────────────────────────────┘
```

## 2. Interface Definitions in `src/harness/copilot.rs`

```rust
pub const COPILOT_RESUME_COMMAND: &str = "ce-ai workflow resume --json";

pub fn has_session_start_hook(hooks_path: &Path) -> bool;
pub fn ensure_session_start_hook(hooks_path: &Path) -> Result<bool, CeError>;
pub fn remove_session_start_hook(hooks_path: &Path) -> Result<bool, CeError>;
```

### Hook Schema in `.github/hooks/hooks.json`:
```json
{
  "version": 1,
  "hooks": {
    "sessionStart": [
      {
        "type": "command",
        "bash": "ce-ai workflow resume --json",
        "powershell": "ce-ai workflow resume --json",
        "timeoutSec": 15
      }
    ]
  }
}
```

## 3. Workflow Action Enhancement in `src/commands/workflow.rs`

In `Action::Resume { json }`:
```rust
Action::Resume { json } => {
    if *json {
        let state_path = ctx.config_dir.join("state.json");
        let state = State::load(&state_path)?;
        let wf = state.current_workflow();
        let repo_state = probe_repo_state(ctx, &wf);
        let openspec_info = repo_state.openspec_context.clone();
        let text_lines = resume_lines(ctx)?;
        let additional_context = text_lines.join("\n");
        let payload = json!({
            "additionalContext": additional_context,
            "workflow": wf,
            "repo_state": repo_state,
            "openspec_context": openspec_info,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        for line in resume_lines(ctx)? {
            println!("{line}");
        }
    }
}
```

## 4. Integration Wiring Points

1. `init_prj.rs:318-325`:
   When `.github` or `copilot-instructions.md` exists:
   Ensure `.github/hooks/hooks.json` has the hook via `crate::harness::copilot::ensure_session_start_hook`.
2. `deinit_prj.rs`:
   Clean up `.github/hooks/hooks.json` via `crate::harness::copilot::remove_session_start_hook`.
3. `doctor.rs`:
   Audit adopted projects with `.github` to ensure `hooks.json` contains `sessionStart`.
