# Proposal: Context-Exhaustion Resilience & Deterministic Workflow Invariants

## Problem
In long AI agent sessions, context window compaction and token dilution degrade LLM recall of multi-page markdown rules. Expecting AI agents to reliably remember governance constraints (such as "never push directly to main" or "wait for 100% green CI matrix status checks") purely from context memory causes catastrophic failure modes when context resets.

## Solution
Move critical workflow invariants out of context memory and into **deterministic platform boundaries**:
1. Enforce GitHub branch protection rules on `main` via `gh api` and CLI tooling (`ce-ai doctor`).
2. Compact `AGENTS.md` into a concise, high-density **Hard-Gate Invariant Index** (~25 lines) at the top of the file, delegating deep manuals to modular skills and `docs/`.
3. Add `ce-ai doctor` health probes to detect missing branch protection, unconfigured git hooks, and invariant index SHA drift.

## In Scope
- `ce-ai doctor` diagnostic checks for GitHub branch protection & local git hooks.
- Scripted branch protection setup helper (`scripts/protect-branch.sh`).
- Compact, high-density `AGENTS.md` invariant index block.
- Full CLI integration tests in `tests/cli.rs`.

## Out of Scope
- Modifying third-party git remote server binaries.
- Hardware key / YubiKey commit signing requirements.

## Risk Analysis & Mitigations
- **R1 (Admin Override)**: Repositories allow admin bypass; mitigated by `ce-ai doctor` auditing branch protection status and reporting drift.
- **R2 (Hook Disabling)**: Local git hooks can be skipped with `--no-verify`; mitigated by treating CI and GitHub branch protection as authoritative enforcement boundaries.

## Success Criteria
- `ce-ai doctor` alerts when branch protection or git hooks are missing.
- Direct git push to `main` fails closed.
- `cargo fmt`, `cargo clippy`, and `cargo test` pass 100% clean across Linux, macOS, and Windows.
