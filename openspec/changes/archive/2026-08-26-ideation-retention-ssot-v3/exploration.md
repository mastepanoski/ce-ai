# Exploration: Ideation Artifact Retention

Origin: `docs/brainstorms/2026-08-25-ideation-retention-ssot-clarification-requirements.md`
(brainstorm evaluated three approaches with the user; decision recorded there — summarized
here for contract completeness).

## Options Evaluated

1. **Retention-by-default reframe (chosen)** — clarify that "disposable" means
   *not-maintained-in-sync, never delete*; keep ideation docs as the permanent raw-history
   layer OpenSpec intentionally does not duplicate.
   - Pros: zero new machinery, zero recurring tokens, all archived-spec references stay valid,
     git history already makes any future deletion reversible.
   - Cons: relies on convention (mitigated: reading `AGENTS.md`/the block is mandatory).

2. **Distill-then-delete protocol** — verification gate + reference rewrite before deletion.
   - Rejected: ceremony and token cost per deletion; risk of lossy distillation; more process
     debt than the problem it solves.

3. **Archive subfolder on change archival** — move superseded ideation docs to an archive dir.
   - Rejected: breaks existing relative references or forces mass edits; cosmetic value only.

## Precedent

The v2 adoption-block change (`openspec/changes/archive/adoption-block-ssot-v2/`) shipped a
content-only wording change through exactly this path: bump one constant, marker-based
in-place replacement, no migration command, MINOR release (1.5.0). This change follows the
identical shape.

## Key Tradeoff Accepted

Propagation via block bump means already-adopted projects receive the clarified rule only on
`init-prj` re-run; until then `status`/`doctor` show stale-version hints (intended UX per
the staleness-alignment design).
