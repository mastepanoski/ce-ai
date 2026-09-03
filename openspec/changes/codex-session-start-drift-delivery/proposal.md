# Proposal: Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI via Native `SessionStart` Hook

## Problem Statement
In `ce-ai v1.31.0`–`v1.33.0`, native Turn-0 synchronization was implemented for Claude Code, OpenCode, and GitHub Copilot CLI. For OpenAI Codex CLI, synchronization currently depends on the textual directive in `AGENTS.md` and `.codex/AGENTS.md`.

OpenAI Codex CLI natively supports lifecycle hooks defined in `<repo>/.codex/config.toml` (and `~/.codex/config.toml`). Official documentation confirms that:
1. `SessionStart` hooks run when a session starts, resumes, or is compacted (`source: "compact"`).
2. Plain text stdout from `SessionStart` hooks is automatically added as extra developer context before the model generates its response.
3. Automated compaction in the middle of a turn executes `SessionStart` hooks matching `source: "compact"`, delivering fresh context immediately to the prompt continuation.

By automating the injection of a `SessionStart` hook into `.codex/config.toml`, `ce-ai` can guarantee 0-step drift recovery and context compaction resilience for OpenAI Codex CLI deterministically.

## In-Scope
1. Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/codex.rs`.
2. Format the hook in `.codex/config.toml` under `[[hooks.SessionStart]]` with `matcher = "startup|resume|compact"` executing `ce-ai workflow resume`.
3. Add `hookSpecificOutput` to `ce-ai workflow resume --json` in `src/commands/workflow.rs` for full schema support.
4. Wire hook injection in `src/commands/init_prj.rs` when `.codex/` is present.
5. Wire hook removal in `src/commands/deinit_prj.rs`.
6. Wire diagnostic finding in `src/commands/doctor.rs`.
7. Unit tests in `src/harness/tests/codex.rs` and CLI integration tests in `tests/cli.rs`.
8. Documentation updates in `zero-step-drift-recovery-explained.md` and `Cargo.toml`/`CHANGELOG.md`.

## Out-of-Scope
- Hooks for unsupported harnesses.

## Risk Evaluation & Mitigation
- **Risk:** Clobbering user settings in `.codex/config.toml`.
  - *Mitigation:* Surgical TOML table manipulation preserves all user-configured MCP servers, env vars, other hook tables, and comments.
