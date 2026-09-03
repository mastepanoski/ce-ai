# Brainstorming: Guaranteed Turn-0 Drift Delivery for Pi Coding Agent via Native Extension

## 1. Problem Statement
In `ce-ai v1.31.0`–`v1.34.0`, native Turn-0 synchronization was delivered for Claude Code, OpenCode, GitHub Copilot CLI, and OpenAI Codex CLI. For Mario Zechner's Pi coding agent (`pi.dev`), synchronization currently relies solely on textual directives in `AGENTS.md` and `.pi/AGENTS.md`.

Pi has an extensible TypeScript/JavaScript architecture based on in-process extensions discovered from `.pi/extensions/*.ts` (project-local) or `~/.pi/agent/extensions/*.ts` (user-global).
Pi lifecycle hooks include:
- `session_start`: Fires when a session is initialized or switched (e.g. `/resume`, `/new`, `/fork`). Useful for resetting state.
- `before_agent_start`: Fires before every agent turn (LLM prompt loop), allowing extensions to dynamically return `{ systemPrompt: ... }` to inject context directly into the agent's prompt context.

## 2. In-Scope vs Out-of-Scope
### In-Scope
1. Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/pi.rs`.
2. Generate the canonical extension at `.pi/extensions/compound-engineering.ts` using `before_agent_start` to inject `ce-ai workflow resume` output into `systemPrompt`.
3. Handle session resets via `session_start` to re-trigger synchronization on `/resume` or `/fork`.
4. Fail-open error handling in the extension to ensure zero agent disruptions if `ce-ai` encounters issues.
5. Wire extension generation in `src/commands/init_prj.rs` when `.pi/` is present.
6. Wire extension removal in `src/commands/deinit_prj.rs`.
7. Wire audit finding in `src/commands/doctor.rs`.
8. Unit tests in `src/harness/tests/pi.rs` and CLI integration test in `tests/cli.rs`.
9. Update `docs/user-guide/zero-step-drift-recovery-explained.md` and bump version to `1.35.0`.

### Out-of-Scope
- Other harnesses (Cursor, Grok, Kimi, etc.) handled in future turns.
