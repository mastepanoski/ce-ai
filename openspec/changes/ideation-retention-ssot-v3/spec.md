# Spec: Ideation Artifact Retention — SSOT Clarification

Origin: `docs/brainstorms/2026-08-25-ideation-retention-ssot-clarification-requirements.md`
(R1–R6, AE1–AE3)

## Requirements

### R1: Retention-default wording (root rule)

WHEN an agent or contributor reads the Single Source of Truth Rule in root `AGENTS.md`,
THEN the rule MUST state that ideation artifacts are retained by default as the permanent
raw-history record, and MUST NOT be interpretable as a deletion mandate.

### R2: Existing instructions preserved

WHEN the clarified rule is read on any surface,
THEN the distill-conclusions, reference-source-doc, and skip-ideation-skills instructions
MUST remain present and unchanged in intent.

### R3: Both managed-block surfaces reworded

WHEN the managed block is rendered for tier `full` or `orchestrator`,
THEN each surface carrying "disposable" wording MUST include the retention clarification.

### R4: Version bump and migration-free upgrade

WHEN `BLOCK_VERSION` is bumped to 3 and `init-prj` re-runs on a project adopted at v2,
THEN the block between markers MUST be replaced with the v3 block, `state.json.block_version`
MUST equal 3, pre-existing surrounding content and line-ending style MUST be preserved, and
no migration command MUST be required.

### R5: Minimal tier untouched

WHEN tier `minimal` renders,
THEN its block body MUST remain byte-identical to the v2-era body, and no runtime logic
outside block text and the version constant MAY change.

### R6: Documentation consistency

WHEN a user reads any guide describing the managed block or restating the SSOT concept
(`project-adoption-guide.md`, `quick-start-workflow-guide.md`,
`fsm-and-checkpoints-explained.md`),
THEN the described behavior and version examples MUST reflect retention-by-default and v3.

## Acceptance Criteria

- AC1 (origin AE1): Given a distilled brainstorm, when an agent considers cleanup after
  reading the rule, then it concludes retention is the default and deletion is neither
  prescribed nor forbidden.
- AC2 (origin AE2): Given an adopted project at block v=2, when `init-prj --tier full`
  re-runs post-upgrade, then the block shows `v=3` with retention wording and
  `block_version == 3` in state.
- AC3 (origin AE3): Given tier `minimal`, when any `init-prj` run completes, then the
  rendered block body is byte-identical to the v2-era body.
