# Design: Codex and Multi-Harness Stop Hook Clean JSON Output

## System Architecture

```mermaid
flowchart TD
    A["Harness Lifecycle Hook / User Invocations"] --> B["ce-ai workflow resume"]
    B --> C{"Check Invocation Mode"}
    C -->|"--pre-invocation"| D["handle_pre_invocation (Antigravity)"]
    C -->|"--event Stop or stdin hook_event_name == 'Stop'"| E["Stop Hook Handler"]
    C -->|"--event PreCompact or stdin hook_event_name == 'PreCompact'"| F["PreCompact Hook Handler"]
    C -->|"--event SessionStart or stdin hook_event_name == 'SessionStart'"| G["SessionStart Hook Handler"]
    C -->|"Interactive Terminal / Standard Pipe"| H["Default Resume Handler"]

    E --> E1["Check stop_hook_active"]
    E1 -->|true| E2["Emit '{}' & Exit 0"]
    E1 -->|false| E3["maybe_auto_checkpoint"]
    E3 --> E2

    F --> F1["maybe_auto_checkpoint"]
    F1 --> E2

    G --> G1["maybe_auto_checkpoint"]
    G1 --> G2{"--json flag or hook context"}
    G2 -->|yes| G3["Emit SessionStart JSON payload"]
    G2 -->|no| G4["Emit Human Resume Lines"]

    H --> H1["maybe_auto_checkpoint"]
    H1 --> H2{"--json flag"}
    H2 -->|yes| G3
    H2 -->|no| G4
```

## Data Structures & CLI Schemas

### 1. `Action::Resume` Arguments in `src/commands/workflow.rs`
```rust
    Resume {
        /// Output machine-readable JSON format.
        #[arg(long)]
        json: bool,
        /// Antigravity PreInvocation hook mode (reads stdin, dedupes per conversationId, injects ephemeralMessage).
        #[arg(long)]
        pre_invocation: bool,
        /// Execution mode override: auto, organic (odd), compound (ce).
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
        /// Explicit lifecycle hook event (e.g., SessionStart, Stop, PreCompact).
        #[arg(long, value_name = "EVENT")]
        event: Option<String>,
    },
```

### 2. Hook Payload Deserializer
```rust
#[derive(serde::Deserialize, Default, Debug)]
struct HookPayload {
    #[serde(alias = "hook_event_name", alias = "hookEventName")]
    hook_event_name: Option<String>,
    #[serde(alias = "stop_hook_active", alias = "stopHookActive")]
    stop_hook_active: Option<bool>,
    #[serde(alias = "session_id", alias = "sessionId")]
    session_id: Option<String>,
}
```

### 3. Event Resolution Logic
```rust
fn resolve_hook_context(
    cli_event: Option<&str>,
) -> (Option<String>, Option<bool>) {
    if let Some(ev) = cli_event {
        return (Some(ev.to_string()), None);
    }

    if !std::io::stdin().is_terminal() {
        let mut buffer = String::new();
        if std::io::stdin().read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
            if let Ok(payload) = serde_json::from_str::<HookPayload>(buffer.trim()) {
                if let Some(name) = payload.hook_event_name {
                    return (Some(name), payload.stop_hook_active);
                }
                if payload.stop_hook_active.is_some() {
                    return (Some("Stop".to_string()), payload.stop_hook_active);
                }
            }
        }
    }

    (None, None)
}
```

### 4. Hook Processing Dispatch
When `resolved_event` is:
- `"stop"` (case-insensitive):
  If `stop_hook_active == Some(true)`, print `{}` and return.
  Execute `maybe_auto_checkpoint(ctx, &repo_root, &state_path)`.
  Print `{}` and return.
- `"precompact"` (case-insensitive):
  Execute `maybe_auto_checkpoint(ctx, &repo_root, &state_path)`.
  Print `{}` and return.
- `"sessionstart"` (case-insensitive) or `None`:
  Execute `maybe_auto_checkpoint(ctx, &repo_root, &state_path)`.
  If `--json` is set, print JSON payload.
  Else if resolved from hook stdin, print JSON payload.
  Else print human resume lines.

## Harness Adapter Contracts
In `src/harness/codex.rs` and `src/harness/claude.rs`:
- `has_codex_event_hook` and `has_event_hook` match both exact command strings and commands beginning with `ce-ai workflow resume`.
- `remove_session_start_hook` removes any command matching or beginning with `ce-ai workflow resume`.
