# Proposal: Guaranteed Turn-0 Drift Delivery for Pi Coding Agent via Native Extension

## Problem Statement
`ce-ai` provides 0-step Turn-0 `RepoState` synchronization for Claude Code, OpenCode, GitHub Copilot CLI, and OpenAI Codex CLI via native lifecycle mechanisms. For Mario Zechner's Pi coding agent (`pi.dev`), synchronization currently relies solely on prompt-based directives in `.pi/AGENTS.md`.

Pi provides an in-process extension system where TypeScript/JavaScript files placed in `.pi/extensions/` are automatically loaded via internal `jiti` transpilation.
By deploying a dedicated extension subscribing to `session_start` and `before_agent_start`, `ce-ai` can guarantee that `ce-ai workflow resume` is executed on Turn-0 and on session resume/fork, appending live `RepoState` directly to `systemPrompt` before the LLM begins its response loop.

## In-Scope
1. Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/pi.rs`.
2. Generate `.pi/extensions/compound-engineering.ts` with Turn-0 caching, `session_start` cache reset, and fail-open execution.
3. Wire extension deployment in `src/commands/init_prj.rs` when `.pi/` exists.
4. Wire extension cleanup in `src/commands/deinit_prj.rs`.
5. Wire diagnostic finding in `src/commands/doctor.rs`.
6. Unit tests in `src/harness/tests/pi.rs` and CLI integration test in `tests/cli.rs`.
7. Update `docs/user-guide/zero-step-drift-recovery-explained.md` and bump version to `1.35.0`.

## Out-of-Scope
- Unsupported harness hooks.

## Risk Evaluation & Mitigation
- **Risk:** Pi fails to load the extension if Node/Bun built-ins are unavailable.
  - *Mitigation:* Uses only `node:child_process` `execSync` built into Node/Bun. Wraps execution in `try / catch` so the agent continues normally even if `ce-ai` fails or times out.
