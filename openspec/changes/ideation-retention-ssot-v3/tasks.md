# Tasks: Ideation Artifact Retention — SSOT Clarification (Block v3)

Work-unit changed-line estimates per CONTRIBUTING.md §4 (~200 LOC target; all units land
well under).

## T1 — OpenSpec contract (this folder)

- [x] Author proposal.md, exploration.md, design.md, spec.md, tasks.md distilled from the
  origin brainstorm. (~150 LOC, docs-only)

## T2 — Reword surfaces, bump version, update pinned tests

- [x] Append retention clarification to `AGENTS.md` SSOT rule; reword both managed-block
  surfaces in `src/commands/init_prj.rs`; bump `BLOCK_VERSION` to 3. (~20 LOC)
- [x] Update literal `v=2` pins to `v=3` in the five enumerated tests; update content pins
  (`init_prj_full_tier_contains_ssot_rule`, orchestrator count==1); retarget and rename
  `doctor_reports_generic_drift_for_tampered_v2_body`. (~40 LOC)
- [x] Add v2→v3 upgrade integration test preserving provenance (`created_file`) and
  line-ending style. (~60 LOC)
- [x] Verification: `cargo test`, clippy `-D warnings`, grep sweep shows no live-template
  `v=2` literals.

## T3 — Documentation alignment

- [x] `docs/user-guide/project-adoption-guide.md`: v3 example header, upgrade section,
  tier rows. (~15 LOC)
- [x] `docs/user-guide/quick-start-workflow-guide.md`: realign three mentions. (~6 LOC)
- [x] `docs/user-guide/fsm-and-checkpoints-explained.md`: realign one mention. (~3 LOC)
- [x] Verification: grep "disposable" across `docs/` returns only retention-consistent
  usages outside whitelisted historical artifacts (docs/solutions/, archived openspec,
  CHANGELOG history).

## T4 — Changelog and version

- [x] `Cargo.toml`: MINOR bump per Key Technical Decisions (precedent: v2 → 1.5.0). (~1 LOC)
- [x] `CHANGELOG.md`: new MINOR heading + entry noting stale-version hints for existing
  adoptions until re-run. (~12 LOC)

## TDD notes

- Write the new v2→v3 upgrade test against the pre-bump tree first where practical
  (asserts current v2 behavior), then flip the constant and watch it drive the v3
  assertions — the failing-then-passing sequence validates the splice path end-to-end.
- The retargeted tamper-header test must keep passing its generic-drift assertion after the
  rename; a stale-classification result there means the fixture literal was not updated.
