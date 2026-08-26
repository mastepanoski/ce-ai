# Design: Ideation Artifact Retention — Adoption Block v3

## Surfaces and Mechanics

| Surface | Location | Change |
|---|---|---|
| Root rule | `AGENTS.md` "Single Source of Truth Rule" section | Append retention clarification |
| `full` tier block | `src/commands/init_prj.rs` `render_block_content` SSOT paragraph | Same clarification, block register |
| `orchestrator` tier block | same file, delegation line list | Extend the disposable-inputs line |
| Version | `pub const BLOCK_VERSION` (same file, ~line 20) | `2` → `3` |

- The constant feeds both the on-disk header (`v={BLOCK_VERSION}`) and
  `state.json.block_version`; body SHA recomputes from content. Never hand-edit hashes.
- `src/harness/custom.rs::render_full_block()` shares the constants — inherits automatically.
- Marker splice + SHA comparison makes re-adoption version-agnostic; no migration command.

## Wording (frozen for implementation; final polish allowed while editing)

Root `AGENTS.md`, appended after "...never maintain brainstorm/ideation documents in sync
with OpenSpec — that duplicates work and burns tokens.":

> Ideation artifacts are retained by default as the permanent raw-history record that
> OpenSpec deliberately does not duplicate — "disposable" never means deleting them; removal
> is an ordinary reversible git decision, never a workflow step.

`full` tier paragraph, closing sentence:

> Ideation artifacts are retained by default as the permanent raw-history record OpenSpec
> intentionally does not duplicate; "disposable" never means deleting them.

`orchestrator` tier line:

> Ideation outputs (`docs/brainstorms/`, `docs/ideation/`) are disposable inputs: distill
> them into the specs before delegation; never maintain them in parallel; retain them as raw
> history instead of deleting them.

Existing sentences (distill/reference/skip-rules) are preserved verbatim in intent on every
surface.

## Test Strategy

Update pinned sites in `tests/cli.rs`:
- Literal `v=2` → `v=3`: `init_prj_and_deinit_prj_roundtrip_fresh_repo`,
  `init_prj_preserves_preexisting_content_and_crlf`,
  `init_prj_replaces_v1_block_with_v2_preserving_content_and_crlf`,
  `init_prj_replaces_lf_only_v1_block_preserving_content`,
  `install_custom_with_flags_copies_layout_manifest_state_and_rules_block`.
- Content pins: `init_prj_full_tier_contains_ssot_rule` (retention sentence present AND
  skip-sentence still present); `init_prj_orchestrator_tier_contains_distillation_line_once`
  (updated phrase, `count() == 1`).
- Retarget `doctor_reports_generic_drift_for_tampered_v2_body`: its injected tamper header
  hardcodes `v=2`, which post-bump classifies StaleVersion (2 < 3) — fixture literal moves to
  `v=3` with corrupted SHA so it keeps exercising the generic-drift branch; rename test to
  drop the misleading "v2".
- Unchanged-green guards: `init_prj_minimal_block_matches_v1_bytes` (body byte-parity),
  malformed-block fail-closed test.
- New integration test: v2 fixture adoption → upgrade re-run renders `v=3` block,
  `state.json.block_version == 3`, pre-existing content and CRLF/LF style preserved,
  `created_file` provenance preserved (regression surface documented in
  `docs/solutions/logic-errors/init-prj-created-file-clobber-on-re-adoption-2026-08-22.md`).

## Constraints Honored

- Atomic writes only (`write_atomic`) — no new write paths introduced.
- `state.save()` strictly after filesystem mutations; errors propagate via `?` (see
  `docs/solutions/bugfixes/transactional-error-propagation-and-state-commit.md`).
- No runtime logic changes outside the template strings and the one constant.
