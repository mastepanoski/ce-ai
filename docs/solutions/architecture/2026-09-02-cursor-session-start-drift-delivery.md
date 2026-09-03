---
module: harness::cursor
tags: [cursor, hooks, session-start, zero-step, additional_context, drift-recovery]
problem_type: architecture
---

# Cursor sessionStart Lifecycle Hook & Turn-0 Drift Delivery

## Context & Objectives
Ensure guaranteed Turn-0 `RepoState` drift recovery when working with Cursor (both desktop editor and Cursor CLI v0.45+) by automatically registering a native `sessionStart` lifecycle hook in `.cursor/hooks.json`.

## Technical Investigation & Decisions

### 1. Cursor CLI & Editor Parity
- Prior Cursor releases had limited hook capabilities in CLI mode.
- In Cursor v0.45+, `sessionStart` is supported in both the desktop editor and Cursor CLI, firing upon composer session initialization.

### 2. Context Injection Protocol
- Cursor command hooks communicate over stdio using JSON.
- When `sessionStart` triggers, Cursor passes session information via stdin and reads JSON from stdout.
- The output schema accepts:
  ```json
  {
    "additional_context": "<context to add to initial system prompt>",
    "env": { "<key>": "<value>" }
  }
  ```
- `ce-ai workflow resume --json` outputs both `additionalContext` (camelCase for Copilot) and `additional_context` (snake_case for Cursor), allowing a single unified payload.

### 3. File Location & Schema
- `.cursor/hooks.json` schema:
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
- Handled idempotently with `write_atomic`.
- Preserves pre-existing user hooks (e.g. `preToolUse`, custom scripts) and custom top-level properties.
- Prunes `.cursor/hooks.json` on de-adoption if only our managed entry existed.
