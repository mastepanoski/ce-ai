# Proposal: Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI via Native `sessionStart` Hook

## Problem Statement
In `ce-ai v1.31.0` and `v1.32.0`, native Turn-0 synchronization was delivered for Claude Code and OpenCode. For GitHub Copilot CLI, agents still rely on textual prompt directives in `AGENTS.md` and `.github/copilot-instructions.md`.

GitHub Copilot CLI natively supports repository-level lifecycle hooks in `.github/hooks/*.json`. It fires `sessionStart` upon session initialization and expects JSON on `stdout` containing an `additionalContext` string, which is automatically injected into the agent's context window. Plain text is discarded.

By implementing native hook configuration for Copilot CLI and enriching `ce-ai workflow resume --json` with `additionalContext`, `ce-ai` can guarantee 0-step drift recovery for GitHub Copilot CLI users deterministically.

## In-Scope
1. Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/copilot.rs`.
2. Format hook in `.github/hooks/hooks.json` specifying `type: "command"` with `bash` and `powershell` running `ce-ai workflow resume --json`.
3. Add `"additionalContext"` to the JSON output of `ce-ai workflow resume --json` in `src/commands/workflow.rs`.
4. Wire hook injection in `src/commands/init_prj.rs`, hook removal in `src/commands/deinit_prj.rs`, and audit findings in `src/commands/doctor.rs`.
5. Unit tests in `src/harness/tests/copilot.rs` and CLI integration tests in `tests/cli.rs`.
6. Documentation updates in `zero-step-drift-recovery-explained.md` and `Cargo.toml`/`CHANGELOG.md`.

## Out-of-Scope
- Unsupported harness hooks (handled in subsequent PRs).

## Risk Evaluation & Mitigation
- **Risk:** Existing tooling expecting specific fields from `ce-ai workflow resume --json`.
  - *Mitigation:* The existing `workflow`, `repo_state`, and `openspec_context` fields remain intact; `additionalContext` is added as a top-level string field.
- **Risk:** Overwriting user hooks in `.github/hooks/hooks.json`.
  - *Mitigation:* Surgical array merging preserves all pre-existing hooks and settings.

## Success Criteria
- `ce-ai init-prj` injects the `sessionStart` hook into `.github/hooks/hooks.json`.
- `ce-ai deinit-prj` cleanly removes the hook and cleans up the file if empty.
- `ce-ai doctor` flags missing Copilot hooks when `.github` is present in adopted projects.
- 100% tests passing across Linux, macOS, and Windows.
