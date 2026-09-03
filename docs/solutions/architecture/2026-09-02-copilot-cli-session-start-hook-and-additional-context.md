---
module: harness::copilot
tags:
  - copilot
  - session-start
  - lifecycle-hooks
  - additional-context
  - drift-recovery
problem_type: architecture
---

# GitHub Copilot CLI Turn-0 `sessionStart` Hook and `additionalContext` Injection

## Problem
In `ce-ai v1.31.0` and `v1.32.0`, native Turn-0 synchronization was achieved for Claude Code (via `SessionStart` command hooks in `.claude/settings.json`) and OpenCode (via plugin `session.created` and `experimental.session.compacting` events). For GitHub Copilot CLI, the system previously relied on human memory or LLM compliance with textual directives in `AGENTS.md`.

## Solution & Architecture
GitHub Copilot CLI natively supports repository-level lifecycle hooks placed in `.github/hooks/*.json`.
1. **Hook Configuration:**
   `.github/hooks/hooks.json` declares:
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
2. **Context Injection Protocol:**
   Copilot CLI discards plain text written to stdout by hooks. To inject dynamic state into the conversation context before the agent responds, the hook command must output JSON containing an `additionalContext` string key.
3. **Dual Purpose Output:**
   `ce-ai workflow resume --json` outputs:
   ```json
   {
     "additionalContext": "<formatted renderable resume text>",
     "workflow": { ... },
     "repo_state": { ... },
     "openspec_context": { ... }
   }
   ```
   This satisfies Copilot CLI's context injection contract while preserving complete structured data for automated agents.
4. **Lifecycle Hooks & Auditing:**
   - `ce-ai init-prj`: Calls `ensure_session_start_hook` when `.github/` is present.
   - `ce-ai deinit-prj`: Calls `remove_session_start_hook`, surgically removing `ce-ai` without disturbing user hooks, and pruning empty files and directories.
   - `ce-ai doctor`: Audits adopted projects containing `.github` to confirm the hook is configured.
