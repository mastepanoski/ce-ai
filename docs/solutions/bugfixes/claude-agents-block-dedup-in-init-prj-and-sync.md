---
title: "Claude Code Instruction Deduplication in init-prj and sync"
category: "bugfixes"
module: "src/commands/init_prj.rs"
date: "2026-09-17"
problem_type: "logic_error"
component: "tooling"
severity: "medium"
symptoms:
  - "init-prj injects the managed 7-Stage block twice into CLAUDE.md and AGENTS.md"
  - "Claude Code parses duplicate prompt instructions (~500 duplicate tokens per session)"
  - "Manual removal of duplicate block in CLAUDE.md is undone on subsequent sync"
root_cause: "logic_error"
resolution_type: "code_fix"
applies_when: "When adopting or syncing projects with Claude Code where CLAUDE.md delegates to AGENTS.md."
tags:
  - "claude"
  - "init-prj"
  - "sync"
  - "deduplication"
  - "agents-md"
  - "prompt-caching"
---

# Claude Code Instruction Deduplication in init-prj and sync

## Problem

When adopting a project with `ce-ai init-prj` where Claude Code is present, `ce-ai` injected the managed 7-Stage workflow block (`<!-- ce-ai:block begin ... -->`) into `AGENTS.md` and also injected an identical block (`<!-- CE-AI MANAGED BLOCK BEGIN -->`) into `CLAUDE.md`. Claude Code expands `@AGENTS.md` at runtime, causing every session to parse the 7-stage block twice (~500 duplicate tokens per session across all adopted workspaces).

## Symptoms

- `CLAUDE.md` starts with `@AGENTS.md` and then appends its own duplicate `<!-- CE-AI MANAGED BLOCK BEGIN -->` block.
- Claude Code session token usage is inflated by approximately ~500 tokens on every prompt turn.
- Manually deleting the duplicate block from `CLAUDE.md` is undone by `ce-ai sync` or subsequent `init-prj` runs, which re-injected the block.

## What Didn't Work

- **Manual deletion**: Removing the duplicate block manually from `CLAUDE.md` fails because `ce-ai sync` detects the harness `.claude/` directory and re-runs `update_claude_md`, re-inserting the block on the next sync.
- **Naive presence check**: Checking only whether `AGENTS.md` exists is insufficient, because projects whose `CLAUDE.md` does *not* import `AGENTS.md` (e.g. customized legacy configs or non-delegating setups) still require direct injection into `CLAUDE.md` to receive workflow directives.

## Solution

1. **Delegation Detection (`src/harness/claude.rs`)**:
   Implemented `delegates_to_agents_md(content: &str) -> bool`, which parses `CLAUDE.md` for `@AGENTS.md`, `@./AGENTS.md`, or `@../AGENTS.md` import syntax while ignoring markdown code fences (```) and list items.

2. **Conditional Hook Reconciliation (`src/commands/init_prj.rs`)**:
   In `reconcile_project_harness_hooks`, before calling `update_claude_md`, `ce-ai` checks if `AGENTS.md` exists and whether `CLAUDE.md` delegates to it:
   - If it delegates: skips appending the managed block and strips any existing legacy duplicate blocks from `CLAUDE.md` via `strip_managed_block`, auto-healing older projects during `ce-ai sync` and `init-prj`.
   - If it does not delegate: preserves the direct injection into `CLAUDE.md`.

3. **Dual Marker Support (`src/harness/claude.rs`)**:
   Enhanced `strip_managed_block` to support both `CE_MANAGED_BEGIN` (`<!-- CE-AI MANAGED BLOCK BEGIN -->`) and `BLOCK_BEGIN_MARKER` (`<!-- ce-ai:block begin`) delimiters.

4. **De-adoption Cleanup (`src/commands/deinit_prj.rs`)**:
   Updated `deinit_prj` to delete `@AGENTS.md` stub files upon project de-adoption.

## Why This Works

By inspecting the semantic relationship between `CLAUDE.md` and `AGENTS.md` rather than assuming they are independent documents, `ce-ai` respects Claude Code's native file-expansion mechanism (`@` import syntax). This eliminates token redundancy while guaranteeing that projects without delegation still receive the full governance block.

## Prevention

- **Multi-File Context Audits**: When managing multiple harness instruction files in a single project, audit whether secondary files import or extend primary files before injecting shared blocks.
- **Idempotent Auto-Healing in Sync**: Lifecycle commands (`sync`, `init-prj`) should clean up obsolete managed blocks when discovering delegation, rather than only operating on clean/new installs.
- **Integration Testing for Delegation**: Add end-to-end tests (such as `init_prj_with_claude_dir_does_not_duplicate_managed_block_into_claude_md` in `tests/cli.rs`) that assert `CLAUDE.md` contains zero duplicate managed blocks when delegating to `AGENTS.md`.

## Related Issues

- GitHub Issue: [#377](https://github.com/mastepanoski/ce-ai/issues/377)
- Pull Request: [#378](https://github.com/mastepanoski/ce-ai/pull/378)
- Architecture: [`docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md`](docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md)
- Architecture: [`docs/solutions/architecture/claude-code-native-harness-adapter.md`](docs/solutions/architecture/claude-code-native-harness-adapter.md)
