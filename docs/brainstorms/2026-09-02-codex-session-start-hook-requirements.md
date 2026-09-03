# Brainstorming: Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI via Native `SessionStart` Hook

## 1. Problem Statement
In `ce-ai v1.31.0`, `v1.32.0`, and `v1.33.0`, native Turn-0 synchronization was implemented for Claude Code, OpenCode, and GitHub Copilot CLI. For OpenAI Codex CLI, synchronization currently depends on the textual directive in `AGENTS.md` and `.codex/AGENTS.md`.

Official OpenAI documentation confirms that OpenAI Codex CLI natively supports lifecycle hooks defined in `<repo>/.codex/config.toml` (and `~/.codex/config.toml`) under the `[[hooks.SessionStart]]` table.
Critically, Codex CLI processes stdout from `SessionStart` hooks:
- **Plain text stdout:** Added directly as extra developer context before the model generates its response.
- **Compaction resilience:** SessionStart hooks matching `source: "compact"` run immediately after automatic or manual context compaction, delivering fresh context to the prompt continuation.

## 2. In-Scope vs Out-of-Scope
### In-Scope
1. Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/codex.rs` targeting `.codex/config.toml`.
2. Support `[[hooks.SessionStart]]` schema in TOML with `matcher = "startup|resume|compact"` and hook handler running `ce-ai workflow resume`.
3. Support `hookSpecificOutput` in `ce-ai workflow resume --json` for enhanced JSON compliance.
4. Wire hook injection in `src/commands/init_prj.rs` when `.codex` exists.
5. Wire hook removal in `src/commands/deinit_prj.rs`.
6. Wire diagnostic finding in `src/commands/doctor.rs`.
7. Unit tests in `src/harness/tests/codex.rs` and CLI integration tests in `tests/cli.rs`.
8. Documentation in `zero-step-drift-recovery-explained.md`.

### Out-of-Scope
- Unsupported harness hooks (handled in subsequent phases).

## 3. Key Design Decisions
- **KD1: TOML Surgical Manipulation:** Use `toml::Table` and `toml::Value` preserving any existing tables, keys, or other hook events (`PreToolUse`, `PostToolUse`, `SessionEnd`).
- **KD2: Cross-Source Matcher:** Match on `"startup|resume|compact"` to guarantee context delivery on fresh startup, resumed sessions, and mid-session token compaction.
