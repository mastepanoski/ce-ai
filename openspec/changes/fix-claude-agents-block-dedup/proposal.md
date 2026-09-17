# Proposal: Skip Managed Block Injection into CLAUDE.md When Delegating to AGENTS.md

## Problem Statement
When a project is adopted via `ce-ai init-prj` or updated via `ce-ai sync`, `ce-ai` writes the managed "🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement" block into `AGENTS.md` (bounded by `<!-- ce-ai:block begin ... -->` and `<!-- ce-ai:block end -->`).

Concurrently, `init_prj` generates a derived stub in `CLAUDE.md` containing `@AGENTS.md\n` so Claude Code loads the directives. However, during project harness reconciliation in `reconcile_project_harness_hooks`:
1. If `.claude/` exists (e.g. `.claude/settings.json`), OR `CLAUDE.md` has any content beyond strictly `"@AGENTS.md"` (such as user-defined instructions or a previously injected block), `reconcile_project_harness_hooks` unconditionally calls `crate::harness::claude::update_claude_md`.
2. `update_claude_md` writes or appends a second demarcated copy of the managed block (bounded by `<!-- CE-AI MANAGED BLOCK BEGIN -->` and `<!-- CE-AI MANAGED BLOCK END -->`) directly into `CLAUDE.md`.

Because Claude Code supports `@` import syntax, loading `CLAUDE.md` imports `AGENTS.md` (which contains the managed block), and then proceeds to evaluate `CLAUDE.md`'s own copy of the block. Every session loading `CLAUDE.md` incurs roughly 500 tokens of duplicate prompt overhead. Furthermore, if a developer manually deletes the duplicate block from `CLAUDE.md`, the next `ce-ai sync` or `ce-ai init-prj` detects `has_user_claude_md = true` and silently re-injects the duplicate block.

## In-Scope
1. **Delegation Detection**: Implement `delegates_to_agents_md(content: &str) -> bool` in `src/harness/claude.rs` to detect when a Claude instruction file delegates to `AGENTS.md` via `@` syntax (e.g., `@AGENTS.md`, `@./AGENTS.md`, `@../AGENTS.md`, respecting code blocks).
2. **Reconciliation Logic Guard**: Update `reconcile_project_harness_hooks` in `src/commands/init_prj.rs` so that when `AGENTS.md` exists and the target Claude markdown file delegates to `AGENTS.md`:
   - It skips calling `update_claude_md`.
   - If the Claude markdown file already contains `CE_MANAGED_BEGIN`, it strips the duplicate managed block using `strip_managed_block`, restoring the file to its clean non-duplicated state.
   - Claude tool/session hooks (`.claude/settings.json`) continue to be configured whenever `.claude/` exists.
3. **De-adoption Integrity**: Ensure `deinit-prj` cleans up `CLAUDE.md` stubs even when they only contain `@AGENTS.md` without `CE_MANAGED_BEGIN`.
4. **Unit & Integration Tests**:
   - Unit tests for `delegates_to_agents_md` covering diverse import syntaxes, comments, whitespace, and code fences.
   - Unit tests for `reconcile_project_harness_hooks` verifying that managed blocks are not injected when `@AGENTS.md` delegation is present, that existing duplicates are stripped, and that non-delegating files still receive the block.
   - Integration tests in `tests/cli.rs` verifying `init-prj` and `sync` behavior with `.claude/` present.

## Out-of-Scope
- Modifying other harnesses that do not have an `@` file-inclusion syntax delegating to `AGENTS.md` (e.g., Cursor MDC rules, Codex TOML, Copilot instructions).
- Altering the content of the managed 7-stage block itself.

## Risk Evaluation & Mitigation
- **Risk (Missing Directives in Claude Code)**: If `CLAUDE.md` does not actually delegate to `AGENTS.md` or `AGENTS.md` does not exist, Claude Code might miss the managed directives.
  - *Mitigation*: We require both `AGENTS.md.exists()` and valid delegation syntax in the Claude file before suppressing injection. If delegation is absent, `update_claude_md` continues to inject the block into `CLAUDE.md`.
- **Risk (User Content Loss in CLAUDE.md)**: Stripping existing duplicate blocks must preserve custom user instructions.
  - *Mitigation*: `strip_managed_block` already isolates `CE_MANAGED_BEGIN...CE_MANAGED_END` and preserves preceding/following text. We add explicit tests verifying user comments and custom rules remain intact.

## Success Criteria
- Running `ce-ai init-prj` on a repository with `.claude/` creates `CLAUDE.md` with only `@AGENTS.md\n` (no duplicate block).
- Running `ce-ai sync` on a project where `CLAUDE.md` had both `@AGENTS.md` and the managed block strips the duplicate block, preserving any user rules.
- Projects without `@AGENTS.md` delegation still receive the managed block in `CLAUDE.md`.
- All tests pass, with 0 clippy warnings.
