---
title: "Multi-Agent Contention Over Shared .git/config During Parallel Worktree Development"
date: "2026-08-22"
category: workflow-issues
module: development-workflow
problem_type: workflow_issue
component: development_workflow
severity: medium
applies_when:
  - "Multiple AI agents (or humans) work concurrently on sibling git worktrees of one repository"
  - "Integration tests read repo-global git config (core.hooksPath, core.bare) and fail non-deterministically"
  - "An agent's changes keep getting reverted by an unseen concurrent writer"
tags:
  - multi-agent
  - git-config
  - worktrees
  - flaky-tests
  - doctor
---

# Multi-Agent Contention Over Shared .git/config During Parallel Worktree Development

## Context

During a parallel multi-agent session on `ce-ai` (sibling worktrees under `<repo>-worktrees/`), agents repeatedly mutated the **shared** `.git/config`:

- `core.hooksPath` flip-flopped between `invalid_hooks` and `.githooks`, making `ce-ai doctor` integration tests fail non-deterministically (`doctor found N finding(s)` only when `invalid_hooks` was active at probe time).
- A concurrent agent briefly set `core.bare=true`; another enabled `extensions.worktreeConfig=true`. Both made `git rev-parse --is-inside-work-tree` return `false` **repo-wide**, breaking worktree detection for every process until manually reverted.

## Guidance

1. **Verify shared-config assumptions before blaming local code.** When a test fails non-deterministically, diff the environment between passing and failing contexts first — including `git config -l --local`. A test asserting on repo-global git config reads whatever the *last* writer left.
2. **When hooks are contended, prove DoD gates manually**: run `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` explicitly (commit with `--no-verify` only if needed, disclosing it). This decouples "is the code correct?" from "which hook config happens to be live right now?".
3. **Tests must self-contain their git fixtures.** A test needing specific git behavior should build it in an isolated temp repo (or sandbox via `GIT_CONFIG_GLOBAL`), never depend on the checkout's global config.
4. **Never reach for `extensions.worktreeConfig` as a workaround** (verified on git 2.55): enabling it disabled inside-work-tree detection repo-wide until reverted — strictly worse than the original problem.
5. **Escalate cross-agent interference to the human coordinator.** Two agents fighting over one `.git/config` cannot be fixed by either alone; serializing shared-resource mutations is a coordination decision.

## Why This Matters

`.git/config` is process-global mutable state within a checkout. One wrong key write invalidated worktree detection for every concurrent tool, CI step, and editor integration — silently. Non-deterministic failures that correlate with timing are almost always shared-state races, not logic bugs.

## When to Apply

- Any test failure that appears/disappears between runs without code changes.
- Before writing a "fix" for flaky tests during parallel sessions.
- When designing tests that read global config (git, env vars, home-dir dotfiles).
- Whenever your changes get reverted by an unseen writer — stop retrying, escalate.

## Examples

- **Diagnosis pattern**: compare `git config core.hooksPath` output in the passing context vs. the failing context; a mismatch points at shared-state interference rather than code.
- **What didn't work #1**: re-setting canonical values (`git config core.hooksPath .githooks`) — the concurrent agent reverted them minutes later (last-writer-wins ping-pong).
- **What didn't work #2**: the per-worktree override attempt described above.
- **What worked**: manual gate runs proving correctness + surfacing the contention to the human coordinator.

## Related

- [Context-Exhaustion Resilience & Deterministic Invariants](../architecture/context-exhaustion-resilience-and-deterministic-invariants.md) — its doctor git-hooks probe carries a caveat cross-linking this doc.
