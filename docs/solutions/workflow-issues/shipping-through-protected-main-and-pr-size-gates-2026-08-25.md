---
title: "Shipping through ce-ai's protected-main and PR-size gates"
date: 2026-08-25
category: workflow-issues
module: github-ci + release-pipeline
problem_type: workflow_issue
component: ci_cd
tags:
  - "ci"
  - "branch-protection"
  - "size-budget"
  - "release"
  - "gh-cli"
---

# Shipping through ce-ai's protected-main and PR-size gates

## Context

Shipping PR #236 (742 changed lines, mostly pipeline-mandated docs) surfaced three gate
behaviors that cost a debug cycle each because they were undocumented: the size-budget
failure mode, the label-vs-rerun trigger gap, and the review-required block on an
all-green PR.

## Guidance

- **PR Size Budget (#108)** counts `git diff --numstat BASE...HEAD` excluding lockfiles
  against the 400-line review boundary. Over the line it demands chained slices OR a
  `size:exception` label. Docs-heavy changes are the natural exception case: pipeline
  artifacts (brainstorm + plan + OpenSpec folder ≈ 600 lines) count toward the budget even
  when executable code is ~20 lines. Always post a justification comment quantifying that
  composition.
- **Adding a label does not re-run checks.** The workflow has no `labeled` trigger; after
  adding `size:exception`, execute `gh run rerun <run-id> --failed` to re-evaluate only the
  failed jobs with the label now visible at checkout time.
- **All-green ≠ mergeable.** Branch protection also requires a review (`REVIEW_REQUIRED`
  blocks even 10/10 green). With `enforce_admins: false`, the owner can complete an
  explicitly ordered merge via `gh pr merge --admin`. If the PR is `BEHIND`, run
  `gh pr update-branch` first — that triggers a fresh full CI run which must go green
  again before merging (strict required-checks).
- **Release flow**: pushing tag `vX.Y.Z` triggers `release.yml` (softprops/action-gh-release)
  which creates the GitHub Release and attaches six platform binaries plus `SHA256SUMS.txt`.
  Curate the auto-generated notes afterwards with `gh release edit vX.Y.Z --notes`.

## Why This Matters

Each of these costs a full CI cycle (~4 min) or a blocked-merge dead end when discovered by
trial and error. Knowing the label/rerun distinction and the BEHIND→BLOCKED sequence turns a
30-minute debugging session into a two-command step — and prevents the temptation to force
merges past genuinely red checks.

## When to Apply

Any PR touching ce-ai that (a) carries mandated documentation artifacts likely to exceed
400 lines, (b) lands against the protected `main` branch, or (c) ships a release tag.

## Examples

```bash
# 1. Size exception path (after justification comment)
gh pr edit <N> --add-label size:exception
gh run rerun <run-id> --failed        # label alone does not re-trigger

# 2. Protected-main path when behind
gh pr update-branch <N> && gh pr checks <N> --watch   # fresh run must pass
gh pr merge <N> --merge --admin                        # owner-ordered merge

# 3. Release
git tag -a v1.24.0 <merge-sha> -m "Release v1.24.0" && git push origin v1.24.0
gh run watch <release-run-id> --exit-status && gh release edit v1.24.0 --notes "..."
```

Related: [adoption-block-version-bump-test-coordination-2026-08-25.md](../test-failures/adoption-block-version-bump-test-coordination-2026-08-25.md)
(the PR whose shipping produced this guidance).
