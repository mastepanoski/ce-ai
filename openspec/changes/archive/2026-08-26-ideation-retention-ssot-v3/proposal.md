# Proposal: Ideation Artifact Retention — SSOT Clarification (Adoption Block v3)

## Problem

The SSOT rule calls ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`)
"disposable inputs" without defining disposal. Because OpenSpec deliberately references
(not copies) those docs, and multiple archived changes carry live links to them, agents
and contributors can reasonably read "disposable" as authorization to delete after
distilling — losing raw history that exists nowhere else and breaking references.

Origin requirements: `docs/brainstorms/2026-08-25-ideation-retention-ssot-clarification-requirements.md`

## In Scope

- Retention-by-default clarification of the SSOT rule wording.
- Surfaces: root `AGENTS.md`, managed block `full` tier paragraph, `orchestrator` tier line.
- `BLOCK_VERSION` bump 2 → 3 (single constant; marker-based re-adoption, no migration).
- Pinned-test updates plus a v2→v3 upgrade-path integration test.
- User-guide alignment: `project-adoption-guide.md`, `quick-start-workflow-guide.md`,
  `fsm-and-checkpoints-explained.md`.
- CHANGELOG entry + MINOR version bump (precedent: v2 shipped as 1.5.0).

## Out of Scope

- Doctor checks for adoption-block drift (deferred since v2).
- Skill definition or FSM stage changes.
- Automated archival/deletion machinery for ideation artifacts.
- The five future process candidates captured in the origin doc.

## Risks

- Existing adoptions surface "stale block version" hints until re-run → release-note callout.
- Missed literal assertions break CI late → enumerated update list + grep sweep (see design).

## Success Criteria

- No reader can interpret "disposable inputs" as a deletion mandate on any of the three surfaces.
- Suite green with v=3 pins; upgrade-path test proves re-adoption preserves provenance.
- Archived-spec references to brainstorms still resolve (nothing deleted).
- Zero ongoing maintenance added (no new checks/scripts/sync steps).
