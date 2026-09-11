# Proposal: Observe-only Ship-readiness gate (Stage 6 + code review)

## Problem
Issue #354: the 7-stage cycle is not enforced at the Ship boundary. An agent moved Verify -> Ship for #351/#352, skipping Stage 6 (`ce-compound`) and `ce-code-review`. The skip was invisible until a human asked, and the unreviewed first attempt contained a real P1. Today `ce-ai workflow resume` reports git/drift/OpenSpec state but never says whether a managed change is actually ready to ship, and it does not surface commits already ahead of base that the checkpoint has not caught up to.

## In Scope (observe-only)
- Ship-readiness evaluation surfaced by `ce-ai workflow resume`/`status`: commits ahead of base, Stage 6 artifact presence (`docs/solutions/**.md` added on the branch), and code-review receipt presence/freshness.
- Code-review receipt recording in `state.json` via a new `ce-ai workflow review-receipt` subcommand (branch-scoped, head-SHA-stamped).
- `ce-ai doctor` non-fatal `doctor-warn:` when an adopted workspace has commits ahead but a Stage 6 or review-receipt gap.
- Resume awareness: report commits ahead of base and warn when the checkpoint stage is earlier than the committed work implies.

## Out of Scope
- **Blocking** PR creation or code writes — that is #334's blocking gate; this change is strictly observe-only (exit code unaffected).
- Verifying that a review actually happened — the receipt records an agent/human assertion, not cryptographic proof.
- Enforcing anything for human-authored PRs or non-adopted workspaces.

## Risks
- False-positive noise: mitigated by only evaluating when the workspace is adopted **and** `commits_ahead > 0`.
- Receipt trust: a receipt can be recorded without a real review; documented as an assertion and recorded with an optional override reason for audit.

## Success Criteria
- `ce-ai workflow resume` prints a Ship Readiness block when commits are ahead, naming Stage 6 and review-receipt gaps.
- `ce-ai workflow review-receipt` records a head-stamped receipt and the next resume reports it present; a new commit makes it stale.
- `ce-ai doctor` emits a non-fatal `doctor-warn:` for the same gaps and still exits per its existing contract.
- No write path is blocked; all commands keep their current exit codes.
