# Proposal: `pr-size-boundaries`

## Why
Issue #108: adopt Gentle AI-style LOC change boundaries as documented,
auditable policy. The 400-line convention existed only implicitly in SDD
task notes; nothing was documented or enforced, and correction loops were
unbounded.

## What Changes
- New CONTRIBUTING.md section: 400-line review boundary, counting contract
  (--numstat, lockfile/generated/binary exclusions), bounded correction
  policy min(200, ceil(original/2)) with one correction per cycle,
  work-unit ~200-line budgets in OpenSpec tasks.md (rescopes narrow only),
  and the size-is-not-risk principle.
- PR template gains a mandatory Changed-Lines Forecast section.
- AGENTS.md Stage-2 checklist references work-unit budgets.
- Documentation-first: CI numstat gate is an explicit fast-follow.
