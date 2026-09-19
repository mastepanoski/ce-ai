# Exploration: Codex and Multi-Harness Stop Hook Clean JSON Output

## Context & Root Cause Analysis

In OpenAI Codex CLI and Anthropic Claude Code, lifecycle hooks are executed as external child processes with event metadata delivered over `stdin` in JSON format:
```json
{
  "hook_event_name": "Stop",
  "session_id": "019283-abc...",
  "transcript_path": "/path/to/transcript.jsonl",
  "cwd": "/path/to/repo",
  "stop_hook_active": false
}
```

When a `Stop` hook completes with exit code 0:
- Codex attempts to parse `stdout` as a JSON object matching its hook return contract.
- Valid responses include `{}` (no-op approval) or `{"decision": "continue", "reason": "..."}` / `{"decision": "block", "reason": "..."}`.
- If the command outputs non-JSON plain text (such as status messages or progress lines), Codex throws:
  ```
  • Hook failed
    └ hook returned invalid stop hook JSON output
  ```

In `ce-ai`, `Action::Resume` in `src/commands/workflow.rs` executes:
1. `let _ = maybe_auto_checkpoint(ctx, &repo_root, &state_path);`
2. If `!json && !pre_invocation`:
   `for line in resume_lines_with_mode(...) { println!("{line}"); }`

Because `SessionStart`, `Stop`, and `PreCompact` hooks were registered with `command = "ce-ai workflow resume"`, the `Stop` event triggered step 2, polluting `stdout` with plain text.

## Evaluated Options

### Option 1: Remove the `Stop` Hook from Codex
- **Pros**: Quick to delete from `src/harness/codex.rs`.
- **Cons**: Completely breaks FSM auto-checkpointing on turn-end for Codex. Workflow stage inference will never run autonomously when the assistant stops, regressing Issue #296 / v1.40.0 capabilities. Violates user product direction: *"Tal vez no hay que eliminar el hook sino para el caso de codex aplicarlo de forma correcta"*.
- **Verdict**: Rejected.

### Option 2: Add `--event Stop` and Rewrite Codex Configs
- **Pros**: Explicit CLI intent.
- **Cons**: Only fixes newly generated `config.toml` files; any existing installation that already ran `ce-ai install` or `init-prj` retains `command = "ce-ai workflow resume"` and continues to fail until manually re-adopted or edited.
- **Verdict**: Incomplete on its own.

### Option 3: Dual Detection (Dynamic Stdin Hook Inspection + Explicit `--event` Flag)
- **Design**:
  1. Add `--event <EVENT>` to `ce-ai workflow resume` for explicit invocation.
  2. If `--event` is not specified, check if `stdin` is not an interactive terminal (`!std::io::stdin().is_terminal()`).
  3. If `stdin` contains a JSON payload with `hook_event_name` (e.g. `"Stop"`, `"PreCompact"`, `"SessionStart"`):
     - Extract the event name and `stop_hook_active` flag.
  4. If the event is `Stop` (or `PreCompact`):
     - If `stop_hook_active` is true, output `{}` and return `Ok(())` (guards against recursive continuation loops).
     - Run `maybe_auto_checkpoint(ctx, &repo_root, &state_path)`.
     - Output `{}` to `stdout` and return `Ok(())`.
  5. If `stdin` is empty, terminal, or non-hook input:
     - Proceed with standard human-readable text lines (or `--json` rehydration payload).
- **Pros**:
  - Automatically heals 100% of existing user installations with zero manual intervention.
  - Supports explicit `--event Stop` invocation.
  - Guarantees zero `stdout` pollution for Codex and Claude Code.
  - Completely preserves interactive terminal developer experience.
- **Verdict**: Selected.
