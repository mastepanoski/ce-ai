# Proposal: Antigravity PreInvocation Turn-0 Drift Delivery

## Problem Statement
Google Antigravity CLI does not possess a literal "session start" lifecycle hook event. However, it provides a native `PreInvocation` hook that fires before the model is invoked and explicitly supports context injection via an `injectSteps` array containing `ephemeralMessage` objects. Because `PreInvocation` fires before *every* model invocation, a naive hook would inject `RepoState` repeatedly on every turn.

We solve this by designing a session-deduplicated `PreInvocation` hook: on the initial turn of a session, it captures the conversation ID from standard input, records a session marker, and injects `RepoState` via `ephemeralMessage`; on subsequent turns of the same session, it acts as a lightweight no-op returning `{}`.

## In-Scope Boundaries
- Add `--pre-invocation` flag to `ce-ai workflow resume` that consumes `stdin`, inspects `conversationId`, evaluates session deduplication markers, and outputs `injectSteps` with `ephemeralMessage`.
- Implement `has_pre_invocation_hook`, `ensure_pre_invocation_hook`, and `remove_pre_invocation_hook` in `src/harness/agy.rs` managing `<project>/.agents/hooks.json`.
- Wire hook installation in `src/commands/init_prj.rs`.
- Wire hook removal and cleanup in `src/commands/deinit_prj.rs`.
- Add diagnostic health probe in `src/commands/doctor.rs`.
- Add unit tests in `src/harness/tests/agy.rs` and CLI integration tests in `tests/cli.rs`.
- Update user documentation in `docs/user-guide/zero-step-drift-recovery-explained.md` explaining the "per-turn with session dedupe" model.

## Out-of-Scope Boundaries
- Modifying Google Antigravity binary or IDE extension internal behavior.
- Ingesting hooks from unmanaged directories outside `.agents/` or `~/.gemini/config/`.

## Risk Evaluation & Mitigation
- **Risk:** Stale session markers causing missing Turn-0 injections in subsequent new sessions.
  - **Mitigation:** Keys markers strictly by unique `conversationId` (`ce-ai-agy-session-<conversationId>.marker`), ensuring each distinct session gets its own fresh Turn-0 injection.
- **Risk:** User config clobbering in `.agents/hooks.json`.
  - **Mitigation:** Parse full JSON tree, insert only under `"compound-engineering".PreInvocation`, preserve all other hooks and keys, and write atomically using `write_atomic`.

## Success Criteria
- `ce-ai init-prj` creates `.agents/hooks.json` containing `PreInvocation` hook for `ce-ai workflow resume --pre-invocation`.
- `ce-ai workflow resume --pre-invocation` injects `RepoState` on turn 0 and returns `{}` on turn 1+.
- `ce-ai doctor` verifies the hook and flags missing configuration with remediation instructions.
- `ce-ai deinit-prj` cleanly cleans up the hook and empty directories.
- 100% test pass rate, green CI matrix, zero clippy warnings.
