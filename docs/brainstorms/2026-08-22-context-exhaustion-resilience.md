# Feature Brief & Requirements: Context-Exhaustion Resilience & Deterministic Workflow Invariants

**Date**: 2026-08-22  
**Status**: Approved (Stage 1 Ideation Complete)  
**Issue Reference**: #97  

---

## 1. Problem Statement

AI coding agents operating in long-running context sessions eventually experience **context window compaction** or **lost-in-the-middle token dilution**. When instructions and critical workflow rules rely entirely on model memory (recall of long prose instructions in `AGENTS.md`), agents tend to skip mandatory gates during long sessions (such as pushing directly to `main` without opening a PR or bypassing CI matrix status checks).

To achieve **verifiable trust** (ISO 42001 / NIST AI RMF), critical delivery gates must be enforced **deterministically at the platform boundary** rather than relying on LLM memory recall.

---

## 2. In Scope

1. **Deterministic Branch Protection & Local Guards**:
   - Automated GitHub repository branch protection configuration script (`scripts/github-branch-protection.sh` / `ce-ai security protect-branch`) enforcing required PRs, 100% green CI matrix status checks, and blocking force pushes to `main`.
   - Local pre-push / pre-commit verification wrapper (`.githooks/pre-push` and `make pr-watch`) preventing direct commits to `main`.
2. **Compact Always-Surviving Invariant Index**:
   - Restructure `AGENTS.md` to feature a compact **Hard-Gate Invariant Index** (~20-25 lines) at the top of the file containing non-negotiable imperative rules.
   - Deep internal explanations and step-by-step guides remain in modular skills and `docs/user-guide/` references.
3. **Observability & `ce-ai doctor` Invariant Probes**:
   - `ce-ai doctor` diagnostic checks verifying:
     - GitHub branch protection status on `main` (via `gh api`).
     - Local git hooks path configuration (`core.hooksPath`).
     - SHA256 integrity of the compact `AGENTS.md` invariant index block.
4. **Persistent Incident & Decision Recording**:
   - Automatic record-keeping of decision trade-offs and root-cause learnings in Engram memory (`mem_save`).

---

## 3. Out of Scope

- Modifying third-party platform API schemas or closed-source LLM context windows.
- Cryptographic hardware-token requirements (FIPS 140-3) for git commit signing.
- Blocking emergency hotfix patches when explicit, post-hoc audited admin overrides are provided.

---

## 4. Success Criteria

1. `ce-ai doctor` warns immediately when `origin/main` lacks GitHub branch protection or when local git hooks are disabled.
2. Direct pushes to `main` fail closed at both the git boundary (`git push origin main`) and the GitHub API boundary.
3. The compact `AGENTS.md` invariant index stays under 30 lines and is automatically verified for SHA integrity by `ce-ai status` and `ce-ai doctor`.
4. 100% green CI test matrix across Linux, macOS, and Windows runners.
