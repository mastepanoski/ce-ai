# Brainstorming: Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI via Native `sessionStart` Hook

## 1. Problem Statement
In `ce-ai v1.31.0` and `v1.32.0`, native Turn-0 lifecycle hooks were added for Claude Code and OpenCode. For GitHub Copilot CLI, synchronization currently relies on the textual directive injected into `AGENTS.md` and `.github/copilot-instructions.md`. If the agent starts a session without running `ce-ai workflow resume`, it operates with stale assumptions regarding git branches, working tree diffs, manifest SHA256 hashes, and OpenSpec task progress.

GitHub Copilot CLI features a first-class repository-level hook system:
- Configuration location: `.github/hooks/*.json` (conventionally `.github/hooks/hooks.json`).
- Lifecycle event: `sessionStart`.
- Execution: Command hook with `bash` and `powershell` keys.
- Context injection mechanism: Copilot CLI reads `stdout` from the hook command; if it is a JSON object with an `additionalContext` key, it automatically injects that text into the agent's context window before the first turn. Non-JSON stdout is discarded.

## 2. In-Scope vs Out-of-Scope Boundaries
### In-Scope
1. Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/copilot.rs` targeting `.github/hooks/hooks.json`.
2. Enhance `ce-ai workflow resume --json` in `src/commands/workflow.rs` to include `"additionalContext": resume_lines(ctx)?.join("\n")`, satisfying Copilot's context injection contract.
3. Wire hook creation into `src/commands/init_prj.rs` when `.github` is present or adopted.
4. Wire hook removal into `src/commands/deinit_prj.rs`.
5. Wire diagnostic audit into `src/commands/doctor.rs`.
6. Unit tests in `src/harness/tests/copilot.rs` and CLI integration test in `tests/cli.rs`.
7. Update documentation in `zero-step-drift-recovery-explained.md`.

### Out-of-Scope
- Unsupported harness APIs (other harnesses will be tackled in dedicated PRs).

## 3. Key Design Decisions
- **KD1: Command Invocation Format:** Use `ce-ai workflow resume --json` with both `bash` and `powershell` fields for cross-platform compatibility across Linux, macOS, and Windows.
- **KD2: Dual Purpose JSON Output:** By adding `additionalContext` to `ce-ai workflow resume --json`, existing tooling reading `"workflow"` or `"repo_state"` is completely unaffected, while Copilot CLI ingests the full formatted status lines.
- **KD3: Surgical JSON Merging:** Preserves all other user-configured hooks (`preToolUse`, `postToolUse`, other `sessionStart` hooks) in `.github/hooks/hooks.json`.
