# Proposal: `workload-guard-citation`

## Why
Precision follow-up to `pr-size-boundaries` after verifying upstream:
the 400-line boundary is enforced by the Review Workload Guard inside
`internal/assets/hermes/sdd-orchestrator.md` (trigger conditions:
`400-line budget risk: High`, `estimated changed lines exceed 400`), with
delivery strategies (`ask-on-risk`, `auto-chain`, `single-pr`, `exception-ok`)
and chain strategies (`stacked-to-main`, `feature-branch-chain`).

## What Changes
CONTRIBUTING.md: cite the orchestrator asset by path and clarify that the
boundary is a review-workload/delivery split signal, never an authoring cap.
