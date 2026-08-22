# OpenSpec Proposal: Worktree Safety Protection & Active Session Preservation

## Executive Summary

AI coding agents often execute automated cleanup commands (`git worktree remove --force`, `git branch -D`) after completing tasks. In multi-agent environments where separate subagents or parallel sessions operate in isolated Git worktrees under `<repo-parent>/<repo-name>-worktrees/`, an over-aggressive cleanup pass by one agent can destroy the working state and uncommitted progress of another active agent.

This change introduces:
1. **Rule 8 in `AGENTS.md` Hard-Gate Invariant Index**: Strictly prohibiting `git worktree remove` or forced deletion of sibling worktrees without explicit user confirmation or verified creation within the current turn.
2. **`ce-ai doctor` Sibling Worktree Health Probe**: Proactively discovering and reporting active sibling worktrees to inform agents and developers of active parallel contexts.

## Problem Statement

When pair programming with AI agents, worktrees are created outside the primary repository checkout (e.g. `../ce-ai-worktrees/model-defaults-tui`). Standard cleanup routines mistake these sibling worktrees for obsolete temporary directories, causing catastrophic data loss when another agent is actively modifying them.

## Success Criteria

- [ ] `AGENTS.md` Hard-Gate Invariant Index updated with Rule 8 (Preserve Active Worktrees).
- [ ] `ce-ai doctor` includes a `worktree` diagnostic probe discovering sibling worktrees in `../<repo>-worktrees/` or `git worktree list`.
- [ ] Integration tests verify `ce-ai doctor` reports sibling worktrees as advisory `doctor-info:` lines.
- [ ] 100% green CI matrix status checks.
