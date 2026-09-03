---
module: harness::codex
tags: [codex, hooks, session-start, compaction, drift-recovery, zero-step]
problem_type: architecture
---

# OpenAI Codex CLI SessionStart Hook & Compaction Resilience

## Problem
In earlier versions of `ce-ai`, Turn-0 `RepoState` synchronization was delivered through automated lifecycle hooks only for Claude Code, OpenCode, and GitHub Copilot CLI. For OpenAI Codex CLI, synchronization relied strictly on textual directives in `AGENTS.md`.

## Solution
1. **Hook Discovery & TOML Schema:**
   OpenAI Codex CLI natively loads lifecycle hooks from `.codex/config.toml` (and `~/.codex/config.toml`).
   We configure `[[hooks.SessionStart]]` with:
   - `matcher = "startup|resume|compact"`
   - `command = "ce-ai workflow resume"`
2. **Context Delivery & Compaction Resilience:**
   Codex CLI captures stdout from `SessionStart` hooks and appends it directly as developer context.
   Critically, by matching `compact`, Codex re-runs the hook immediately following automatic or manual token compaction, guaranteeing fresh `RepoState` even mid-session without human intervention.
3. **Dual Payload Output:**
   `ce-ai workflow resume --json` outputs both plain text stdout and a JSON object containing `hookSpecificOutput`, enabling Codex to ingest state via either format.
4. **Surgical Lifecycle Management:**
   `ensure_session_start_hook` and `remove_session_start_hook` in `src/harness/codex.rs` use `toml::Table` manipulation with `write_atomic`, preserving all existing tables and pruning empty configuration files upon de-adoption.
