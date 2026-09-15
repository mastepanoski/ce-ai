---
module: governance
tags: [governance, security-review, prompt-gates, mechanical-enforcement, fork-prs, code-review]
problem_type: process
title: "Sensitive Surface Fork PR Review: Prompt-Based Guidelines vs. Mechanical Enforcement"
applies_when: "When encountering issues related to sensitive surface fork pr review: prompt-based guidelines vs. mechanical enforcement in security."
---

# Sensitive Surface Fork PR Review: Prompt-Based Guidelines vs. Mechanical Enforcement

## Problem

Pull Request #343 (`fix: static musl Linux releases para compatibilidad cross-distro (GLIBC_2.39 not found)`) was submitted from an external fork (`oTTa/ce-ai`). It modified `.github/workflows/release.yml`, `scripts/install.sh`, and `scripts/release-integrity.sh`, and subsequently required reconciling target triples and fallback logic in `src/source/binary_release.rs`.

In `ce-ai` operational guidelines (and NIST AI RMF governance directives), modifications touching external network endpoints, binary downloading, executable file replacement, and release packaging constitute a **sensitive attack surface**. Under the repository's rules, changes of this nature require an explicit `security-reviewer` pass before merge.

However, PR #343 was squash-merged into `main` with zero review activity recorded on GitHub (`gh pr view 343 --json reviews,comments` returned empty arrays: `reviews: []`, `comments: []`).

Although the code modifications were verified manually and proved technically sound (preserving fail-closed SHA256 integrity and Zip-Slip traversal protections), the process that permitted an external PR touching security-critical infrastructure to merge without a recorded audit trail highlights an operational governance vulnerability.

## Solution

### 1. Root Cause: Prompt Guidelines vs. Mechanical Enforcement

The root cause is a fundamental asymmetry between **advisory prompt rules** and **deterministic mechanical gates**:

- The directive to invoke `security-reviewer` for sensitive surfaces existed only as written guidance in operating directives (`AGENTS.md` and review guidelines).
- There was no automated GitHub Actions check, required status check, or branch protection rule enforcing that a security review receipt or specific label be registered prior to merge.
- Under operational focus—rebasing a fork branch across recent release commits, resolving merge conflicts in lockfiles and changelogs, and aligning CI matrices—operators (human or autonomous) naturally prioritize clearing blocking gates over executing unenforced advisory checklists.

This dynamic directly mirrors the architectural debate in Issues #332, #333, and #334 regarding Stage 4 (`ce-work`) gate checks: behavioral prompts and written policies decay under operational pressure unless backed by deterministic mechanical gates that physically block progression.

### 2. Defense-in-Depth Architecture as a Safeguard

While the review process omitted formal logging, the system did not suffer a security degradation because the underlying codebase implements strict defense-in-depth:

1. **Cryptographic Fail-Closed Semantics**: `src/source/binary_release.rs` enforces SHA256 validation before decompression. Any tampering immediately halts execution with `CeError::Verification` (exit code 6).
2. **Path Traversal Defenses**: Archive extraction enforces strict Zip-Slip protection against `..`, absolute paths, or drive letters.
3. **Atomic Swapping**: Executable replacement uses atomic `rename(2)` on POSIX and NTFS `FILE_SHARE_DELETE` staging on Windows.

Architectural resilience ensured that a procedural gap did not translate into a vulnerability.

### 3. Governance Hardening Recommendations

To prevent unreviewed merges on sensitive surfaces in future workflows:

- **Audit Trail for Fork PRs**: Any PR originating from an external fork that touches sensitive paths (`.github/workflows/`, `scripts/`, `src/source/`, `src/state/`) must produce an explicit, registered review artifact or comment on GitHub before merge approval.
- **Mechanical Gate Exploration**: Evaluate introducing a lightweight CI check (or label requirement like `security:reviewed`) that blocks merging fork PRs touching sensitive paths until verified.
- **Transparency in Status Reports**: Document operational trade-offs explicitly in compounding records rather than leaving governance exceptions unrecorded.

## Key Learnings

1. **Prompt-Based Gates Do Not Prevent Procedural Drift**: Operational directives documented in prompts or markdown files guide behavior, but cannot guarantee compliance without mechanical enforcement in the CI/CD pipeline.
2. **External Fork PRs Demand Heightened Audit Rigor**: Contributions from external forks touching release pipelines, installers, or binary download logic represent supply-chain exposure and must leave an immutable review audit trail on GitHub.
3. **Defense-in-Depth Invariants Mitigate Process Failures**: When operational processes experience lapses, strict compile-time checks, cryptographic verification, and fail-closed runtime error handling prevent technical security breaches.
