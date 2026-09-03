---
module: harness::pi
tags: [pi, extensions, before_agent_start, session_start, drift-recovery, zero-step]
problem_type: architecture
---

# Pi Coding Agent Native Extension & Turn-0 Drift Delivery

## Problem
Prior to `ce-ai v1.35.0`, Turn-0 synchronization for Mario Zechner's Pi coding agent (`pi.dev`) depended solely on text directives in `.pi/AGENTS.md`.

## Solution
1. **In-Process Extension Discovery:**
   Pi automatically discovers, transpiles, and runs TypeScript/JavaScript modules located in `.pi/extensions/*.ts` using internal `jiti` without requiring an explicit compilation step or project `node_modules`.
2. **Lifecycle Coordination (`session_start` + `before_agent_start`):**
   - `session_start` fires on session initialization or switch (`/resume`, `/new`, `/fork`), resetting the internal `sessionInitialized` flag.
   - `before_agent_start` intercepts the prompt before the LLM loop starts. If uninitialized, it executes `ce-ai workflow resume` in `ctx.cwd` and appends the live `RepoState` directly to `event.systemPrompt`.
3. **Fail-Open Error Resilience:**
   Child process execution is wrapped in a `try / catch` block with a 5-second timeout, ensuring agent execution is never halted even if `ce-ai` encounters issues or is absent from `PATH`.
4. **Surgical File Lifecycle:**
   `ensure_session_start_hook` and `remove_session_start_hook` in `src/harness/pi.rs` atomically manage `.pi/extensions/compound-engineering.ts`, preserving any pre-existing user extensions and pruning empty directories upon de-adoption.
