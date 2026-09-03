---
module: harness::grok
tags: [grok, hooks, session-start, zero-step, negative-finding, stdout-discarded]
problem_type: architecture
---

# Grok Build CLI Turn-0 Drift Delivery Evaluation

## Context & Objectives
Investigation of xAI Grok Build CLI (`grok`) to determine whether a native lifecycle hook (`SessionStart` in `~/.grok/hooks/*.json` or `.grok/hooks/*.json`) can deterministically deliver Turn-0 `RepoState` into the conversation context.

## Findings & Architectural Evidence

### 1. Hook Configuration Architecture
- Grok Build CLI provides a hook subsystem where hooks can be configured globally (`~/.grok/hooks/*.json`) or per-project (`<project>/.grok/hooks/*.json`).
- Hooks subscribe to lifecycle events including `SessionStart`, `SessionEnd`, `PreToolUse`, and `UserPromptSubmit`.
- When `SessionStart` fires, the CLI supplies session metadata (`sessionId`, `cwd`, `workspaceRoot`) to hook processes via standard input (`stdin`) in JSON format.

### 2. Discarded Standard Output (`stdout`)
- Unlike Claude Code, OpenAI Codex CLI, GitHub Copilot CLI, and Cursor, **Grok CLI deliberately ignores and discards standard output (`stdout`) from `SessionStart` hooks**.
- Grok enforces an internal runtime policy preventing arbitrary context injection via `stdout` on session initialization.
- Even when a hook outputs JSON with `additionalContext` (matching Claude Code's schema), the Grok CLI does not ingest or append this text into the model's system prompt or conversational context window.

### 3. Control & Logging Semantics
- In Grok CLI, `SessionStart` is strictly a control and telemetry event:
  - Exit code `0`: Indicates success and allows session startup to proceed normally.
  - Exit code `2`: Aborts or blocks execution, printing `stderr` to the user terminal.
- The hook execution protocol does not support bidirectional context exchange.

### 4. Project-Level Security Gating
- Project-level hooks located in `.grok/hooks/*.json` are not executed by default; they require manual user trust via `/hooks-trust` or `--trust` flags.
- Relying on project hooks without user intervention would fail closed in untrusted directories.

## Conclusion & Architectural Decision
Grok Build CLI does not support conversational context injection via `SessionStart` hooks. Attempting to register `ce-ai workflow resume` as a `SessionStart` hook would execute silently while failing to deliver `RepoState` to the LLM.

Grok Build CLI remains governed by the text-based Turn-0 Session Directives in `AGENTS.md` and `.grok/rules/compound-engineering.md` injected during `ce-ai init-prj`.
