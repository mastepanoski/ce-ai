# Proposal: Deterministic Tasks Tail Validator Script Helper

## Problem Statement
In AI-assisted workflows, agents frequently omit critical post-implementation lifecycle steps (code simplification, review receipts, compound learning capture in `docs/solutions/`, and glossary accretion in `CONCEPTS.md`) because:
1. `tasks.md` in OpenSpec changes is drafted by agents with only technical code implementation units, omitting the lifecycle tail.
2. Ship-readiness gaps in `ce-ai` are intentionally observe-only advisories rather than hard blockers.
3. Language-model agents suffer from stochastic execution decay across long sessions, skipping unlisted tasks unless deterministically enforced by fail-closed tooling.

## In-Scope
- Create `scripts/validate-tasks-tail.py`: a standalone, zero-dependency Python 3 script that inspects `tasks.md` files.
- Automatically detect whether a change touches code (`src/**`, `tests/**`) or is documentation/chore-only.
- Verify presence of the 3 required lifecycle items: (1) `ce-simplify-code` / simplification, (2) `ce-code-review` / `review-receipt`, and (3) `ce-compound` (`docs/solutions/` / `CONCEPTS.md`).
- Provide an automated `--fix` flag that appends a standardized `Lifecycle Tail` work unit to `tasks.md` if missing.
- Support `--json` output for automated tooling and agent consumption.
- Add test coverage verifying detection, error reporting, exempt changes, and `--fix` appending idempotence.

## Out-of-Scope
- Modifying the core Rust `ce-ai` binary in this pilot (can be integrated in a follow-up feature once the script contract is proven).

## Success Criteria
- `python3 scripts/validate-tasks-tail.py <path>` fails with exit code 1 when code tasks omit the lifecycle tail.
- `python3 scripts/validate-tasks-tail.py <path> --fix` appends the missing tail and subsequently exits with code 0.
- Documentation-only changes are recognized as exempt.
- All test suites pass (`cargo test`, python script test suite, `make e2e`).
