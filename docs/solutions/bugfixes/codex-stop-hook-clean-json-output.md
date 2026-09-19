---
title: "Codex CLI Stop Hook JSON Validation and Workflow Auto-Checkpoint Clean Output"
category: "bugfixes"
module: "src/commands/workflow.rs"
date: "2026-09-18"
problem_type: "logic_error"
component: "workflow"
severity: "high"
symptoms:
  - "Codex CLI outputs '• Hook failed └ hook returned invalid stop hook JSON output' at turn-end"
  - "Stop hook execution fails when running ce-ai workflow resume"
  - "Autonomous FSM auto-checkpointing disrupted in OpenAI Codex CLI"
root_cause: "logic_error"
resolution_type: "code_fix"
applies_when: "When executing workflow resume from lifecycle hooks (Stop, PreCompact) in OpenAI Codex or Claude Code"
tags:
  - "codex"
  - "claude"
  - "hooks"
  - "stop-hook"
  - "workflow-fsm"
  - "auto-checkpoint"
  - "json-schema"
---

# Codex CLI Stop Hook JSON Validation and Workflow Auto-Checkpoint Clean Output

## Problem

When `ce-ai` configures OpenAI Codex CLI in `~/.codex/config.toml` (and Claude Code in `.claude/settings.json`), it registers lifecycle hooks for `SessionStart`, `Stop`, and `PreCompact` with `command = "ce-ai workflow resume"`. The `Stop` and `PreCompact` hooks were wired to trigger autonomous Workflow FSM stage progression (`maybe_auto_checkpoint`) without operator friction.

However, `ce-ai workflow resume` was designed to emit human-readable status text lines to `stdout` by default. Unlike `SessionStart` (which accepts free text or `hookSpecificOutput`), OpenAI Codex CLI strictly inspects `stdout` of `Stop` lifecycle hooks, requiring valid JSON conforming to its stop-hook schema (e.g. `{}` or `{"decision": "continue"}`). Because `workflow resume` printed human text lines to `stdout`, Codex failed at turn-end with:
```
• Hook failed
  └ hook returned invalid stop hook JSON output
```

## Symptoms

- Every agent turn in Codex CLI ended with a red hook failure: `hook returned invalid stop hook JSON output`.
- Removing the `Stop` hook eliminated the error message but permanently disabled autonomous turn-end workflow progression in Codex, requiring manual checkpoints.
- Attempting to pass `--json` to `workflow resume` previously emitted `SessionStart` specific context with `additionalContext`, which Codex also rejected for `Stop` hooks.

## What Didn't Work

- **Deleting the `Stop` Hook**: While removing `[[hooks.Stop]]` from `~/.codex/config.toml` silences the error, it sacrifices automated FSM state progression. The agent's workflow stage stalls at Stage 1 even as tasks are completed.
- **Generic `--json` flag**: Emitting top-level context fields (`additionalContext`, `workflow`, `repo_state`) fails Codex's `Stop` schema validation, which strictly expects `{}` or a decision payload.

## Solution

1. **Dual Hook Event Detection (`src/commands/workflow.rs`)**:
   - Added explicit `--event <EVENT>` CLI parameter to `ce-ai workflow resume` (e.g. `--event Stop`, `--event PreCompact`).
   - Implemented dynamic `stdin` inspection via `resolve_hook_context`: when executed in a non-interactive terminal environment, `workflow resume` parses `stdin` for `hook_event_name` (e.g. `"Stop"`, `"PreCompact"`, `"SessionStart"`).

2. **Silent Clean JSON Output for Turn-End (`src/commands/workflow.rs`)**:
   - For `Stop` and `PreCompact` hook events, `ce-ai workflow resume` executes `maybe_auto_checkpoint` to progress the Workflow FSM.
   - Emits exactly `{}` to `stdout` and exits cleanly with code 0.
   - Prevents `stdout` pollution while fulfilling the hook schema contract.

3. **Continuation Loop Guard**:
   - Inspects `stop_hook_active`: if `true`, returns `{}` immediately without re-triggering checkpoint side-effects, preventing recursive continuation loops.

4. **Harness Detector Prefix Resiliency (`src/harness/codex.rs`, `src/harness/claude.rs`)**:
   - Updated `has_codex_event_hook` and `has_event_hook` to recognize commands starting with `ce-ai workflow resume`, ensuring full backward and forward compatibility for existing configurations and flagged commands.

## Why This Works

Codex and Claude Code hook runners pipe the event metadata payload into `stdin` of the configured subprocess. By inspecting `stdin` before generating output, `ce-ai workflow resume` transparently distinguishes between interactive developer usage (which requires formatted text banners) and automated lifecycle execution (which requires `{}`). This automatically heals all existing user installations without requiring manual config migrations or re-running `init-prj`.

## Prevention

- **Hook Schema Adherence**: Every lifecycle hook across harnesses (Codex, Claude, Cursor, Copilot, Antigravity) has a distinct `stdout` contract. Turn-end (`Stop`) hooks must always emit `{}` or structured decision JSON, never free text.
- **Non-Interactive Detection**: Subcommands hooked into agent harnesses must check terminal state and `stdin` before printing human-oriented messages.
- **Integration Test Coverage**: Test suites must include tests with piped stdin simulating `{"hook_event_name": "Stop"}` to guarantee zero `stdout` pollution.

## Related PRs & Issues

- Pull Request: [#397](https://github.com/mastepanoski/ce-ai/pull/397) (Fix implementation)
- Pull Request: [#398](https://github.com/mastepanoski/ce-ai/pull/398) (OpenSpec archival)
- Release: `v1.63.1`
- Architecture: [`docs/solutions/architecture/workflow-fsm-auto-checkpoint-lifecycle-and-provenance.md`](docs/solutions/architecture/workflow-fsm-auto-checkpoint-lifecycle-and-provenance.md)
- Architecture: [`docs/solutions/architecture/codex-native-harness-adapter.md`](docs/solutions/architecture/codex-native-harness-adapter.md)
