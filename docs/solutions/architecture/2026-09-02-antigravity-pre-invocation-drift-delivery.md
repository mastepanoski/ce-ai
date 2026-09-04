---
module: harness::agy
tags: [antigravity, agy, hooks, pre-invocation, session-deduplication, ephemeralMessage, zero-step, drift-recovery]
problem_type: architecture
---

# Google Antigravity PreInvocation Hook & Turn-0 Drift Delivery

## Context & Objectives
Ensure guaranteed Turn-0 `RepoState` drift recovery when working with Google Antigravity CLI (`agy`) by registering a native `PreInvocation` lifecycle hook in `.agents/hooks.json` that injects live workspace state before the model's first reasoning turn.

## Technical Investigation & Decisions

### 1. Hook Mechanism & Per-Turn vs Session-Start Distinction
- Official Antigravity hooks specification: `https://antigravity.google/docs/hooks/`.
- Antigravity CLI defines 5 hook events: `PreToolUse`, `PostToolUse`, `PreInvocation`, `PostInvocation`, and `Stop`.
- There is no dedicated single-shot session start event.
- However, `PreInvocation` fires before every model invocation and supports true context injection via `injectSteps` containing `ephemeralMessage` (transient system prompt injection).
- Because `PreInvocation` fires before *every* model invocation, running drift recovery naively on each turn would spam duplicate context and burn tokens.

### 2. Session Deduplication Strategy
- To provide true Turn-0 drift recovery, `ce-ai workflow resume --pre-invocation` implements session deduplication:
  - Antigravity passes invocation context over `stdin` as JSON (`conversationId`, `invocationNum`, etc.).
  - On the first turn (Turn 0), `ce-ai` sanitizes `conversationId` (or fallback `sessionId`), records a session marker in `std::env::temp_dir()`, and returns:
    ```json
    {
      "injectSteps": [
        {
          "ephemeralMessage": "<live RepoState>"
        }
      ]
    }
    ```
  - On subsequent turns within the same session, `ce-ai` detects the existing marker file and immediately emits `{}` in sub-millisecond time.
  - If no conversation identifier is available, it relies on `invocationNum == 0` for Turn-0 gating.

### 3. File Location & Schema
- Managed file: `.agents/hooks.json` (workspace) or `~/.gemini/config/hooks.json` (global).
- Top-level schema:
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
- Mutation invariants:
  - All file mutations use `crate::state::write_atomic`.
  - Non-destructive: preserves existing user hook groups and extra events.
  - Idempotent: re-running `ensure_pre_invocation_hook` avoids duplicate entries.
  - Clean de-init: `remove_pre_invocation_hook` removes only the managed hook, deletes `hooks.json` when empty, and prunes `.agents` if no other files remain.
  - Health probe: `ce-ai doctor` verifies hook presence in adopted `.agents/` projects.
