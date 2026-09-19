# Proposal: Codex and Multi-Harness Stop Hook Clean JSON Output

## Problem Statement
When `ce-ai` registers lifecycle hooks for OpenAI Codex CLI in `~/.codex/config.toml` (and Claude Code in `.claude/settings.json`), it configures `SessionStart`, `Stop`, and `PreCompact` hooks to execute `ce-ai workflow resume`. The primary purpose of wiring `Stop` and `PreCompact` is to trigger autonomous Workflow FSM progression via `maybe_auto_checkpoint` at turn-end and before compaction without operator friction.

However, `ce-ai workflow resume` defaults to outputting human-readable plain text lines (e.g., `workflow: resuming execution from latest checkpoint...`, status headers, and environment state). OpenAI Codex CLI strictly inspects `stdout` of `Stop` lifecycle hooks, requiring either an empty response or valid JSON conforming to the hook response schema (e.g., `{}` or `{"decision": "continue"}`). When `workflow resume` emits human-readable text, Codex halts with:
```
• Hook failed
  └ hook returned invalid stop hook JSON output
```

Eliminating the `Stop` hook from `config.toml` avoids the error but sacrifices autonomous turn-end stage inference and auto-checkpointing for Codex users. Instead, `ce-ai workflow resume` must correctly handle `Stop` and `PreCompact` hook execution by performing auto-checkpointing silently and emitting clean, valid JSON (`{}`) to `stdout`.

## In-Scope
1. **Lifecycle Hook Event Protocol in `workflow resume`**:
   - Add explicit `--event <EVENT>` CLI argument to `ce-ai workflow resume`.
   - Implement transparent hook detection on `stdin` when not running in an interactive terminal.
   - For `Stop` and `PreCompact` events: execute `maybe_auto_checkpoint` and emit `{}` to `stdout` with exit code 0.
   - For `SessionStart` event: preserve rich re-hydration context while ensuring hook JSON compliance when requested or detected.
   - Support `stop_hook_active` flag handling to prevent continuation loops.
2. **Harness Compatibility**:
   - Ensure `src/harness/codex.rs` and `src/harness/claude.rs` hook detectors (`has_session_start_hook`, `remove_session_start_hook`) remain robust for both default `ce-ai workflow resume` and flagged invocations.
3. **Comprehensive Test Suite**:
   - Add unit and integration tests simulating `Stop`, `PreCompact`, and `SessionStart` invocations via `stdin` and `--event` flags.
   - Verify zero non-JSON stdout pollution when `Stop` is executed.
4. **Quality Gates & Release**:
   - Pass all formatting, linting, unit tests, and DoD requirements.
   - Bump SemVer patch to `v1.63.1` and update `CHANGELOG.md`.

## Out-of-Scope
1. Modifying Codex CLI binary internals or OpenAI hook runtime behaviors.
2. Altering unrelated harness adapters that do not execute `workflow resume` on `Stop`.
3. Changing the stage inference rules in `infer_stage_from_repo`.

## Risk Evaluation & Mitigation
- **Risk (Terminal UX Regression)**: Developers running `ce-ai workflow resume` manually in an interactive terminal might have their human-readable output suppressed.
  - *Mitigation*: Terminal detection (`std::io::stdin().is_terminal()`) guarantees that interactive terminal users always receive standard human-readable text lines unless `--event Stop` or `--json` is explicitly requested.
- **Risk (Broken Existing User Configs)**: Users who already adopted `ce-ai` with `~/.codex/config.toml` having `command = "ce-ai workflow resume"` might continue to fail if the command string was required to change.
  - *Mitigation*: Automatic `stdin` payload inspection dynamically detects `hook_event_name: "Stop"` from the incoming stream, immediately repairing existing user installations without requiring manual config migrations.
