---
title: "Observe-Only Gate Signal Integrity"
category: "architecture"
module: "src/commands/"
tags: ["observe-only", "gate", "signal-integrity", "git-diff", "sha-identity", "architecture"]
problem_type: "bug"
severity: "P2"
---

# Observe-Only Gate Signal Integrity

## Problem
The first implementation of the observe-only Ship-readiness gate (issue #354) was non-blocking and non-mutating, but a code review found that its **signals lied** in several ways. A gate that warns is only useful if its warnings are trustworthy; false "ready" and false "gap" signals are worse than no gate.

Defects found and fixed:
1. **Identity mismatch** — receipts stored/compared short vs full SHAs, so a full-SHA `--head` produced a permanent false `ReviewReceiptStale`; an unresolvable head silently stored `"unknown"`.
2. **Deletion counted as presence** — `git diff --name-only` lists deletions, so a branch that only *removed* a `docs/solutions/**.md` file was scored as "Stage 6 present" → false ready.
3. **Fail-open base resolution** — only `origin/main`/`main` were tried; on `master`/`trunk`/no-remote repos `commits_ahead` silently became `0` and the whole gate vanished, indistinguishable from ready.
4. **Unscoped noise** — the gate ran on any git repo ahead of base, not just adopted workspaces, producing false-positive warnings the design explicitly promised to avoid.
5. **Duplicated warnings** — `resume` nested `status` (which emitted gap warnings) and then emitted the same gaps again in its own block.

## Root Cause
The gate was built from convenient git/short-SHA signals without asking two questions per signal: *does this measure the thing I claim?* and *what does this look like when the input is missing, deleted, or unscoped?* Concretely:
- Short SHAs are not stable identities; abbreviate-length changes and user-supplied full SHAs do not compare equal.
- `--name-only` includes `D` (delete) entries; only `--diff-filter=ACMR` answers "was an artifact added/modified?".
- A silent `return 0` on unresolved base is fail-open: absence of evidence rendered as evidence of readiness.
- "Observe-only" was read as "run everywhere", ignoring the adopted-workspace scope that makes it meaningful.
- Nested output functions each rendered the same warnings independently.

## Solution
1. **Normalize identity at the boundary.** Resolve every commit-ish through `git rev-parse --verify <spec>^{commit}` to the full 40-char SHA, store the full SHA, and compare full-to-full. Reject unresolvable heads with `CeError::Usage` instead of storing a placeholder.
```rust
fn resolve_commit_sha(repo_root: &Path, input: Option<&str>) -> Option<String> {
    let rev = format!("{}^{{commit}}", input.unwrap_or("HEAD"));
    let out = git_probe(repo_root, &["rev-parse", "--verify", "--quiet", &rev])?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string()).filter(|s| !s.is_empty())
}
```
2. **Distinguish additions from deletions** with `git diff --name-only --diff-filter=ACMR`; for dirty files, require the path to still exist on disk.
3. **Enumerate base fallbacks** (`origin/main`, `main`, `origin/master`, `master`, `@{upstream}`) and treat unresolved base as "not evaluated" rather than "ready".
4. **Gate on scope**: evaluate only for adopted workspaces (`adoption_status.is_some()` / `State::is_project_adopted`).
5. **Single owner per warning**: `status_lines` owns gap warnings; the resume information block reports signals only.

## Prevention
- For any gate/preview, ask "what is this signal when the input is missing, deleted, renamed, or unscoped?" — test those arms explicitly.
- Never use short SHAs as identity keys; normalize to full SHAs at the boundary.
- `git diff --name-only` is not "files added". Use `--diff-filter` when counting artifacts.
- Fail-safe, not fail-open: an unresolved prerequisite must produce an explicit "not evaluated" state, never a silent pass/absence.
- Scope a gate before it warns: the cheapest way to kill a gate's credibility is false positives on unrelated repos.
- One function owns each user-facing warning; nested renderers must not re-emit it.
- Test the integration output, not just the pure helpers — the duplicated-warning regression hid precisely in the untested `resume` composition.
