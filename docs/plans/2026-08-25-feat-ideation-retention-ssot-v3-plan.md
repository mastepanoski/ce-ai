---
title: "feat: Ideation artifact retention — SSOT clarification (adoption block v3)"
type: feat
status: completed
date: 2026-08-25
origin: docs/brainstorms/2026-08-25-ideation-retention-ssot-clarification-requirements.md
---

# feat: Ideation artifact retention — SSOT clarification (adoption block v3)

## Summary

Reword the Single Source of Truth rule so ideation artifacts are explicitly *retained by default* ("disposable" never means delete), applying the same clarification to root `AGENTS.md` and both managed-block surfaces, and bump the adoption block to v=3 via the single `BLOCK_VERSION` constant so marker-based re-adoption propagates everything with zero migration.

---

## Problem Frame

The current "disposable inputs" wording can be read as authorization to delete `docs/brainstorms/` / `docs/ideation/` once distilled — losing raw history that OpenSpec deliberately does not duplicate and breaking live references from archived specs. Full situation and actor map: see origin document.

---

## Requirements

Trace to origin R-IDs:

- R1. SSOT rule states ideation artifacts are retained by default as the permanent raw-history layer; "disposable" means *not maintained in sync, never treated as specs* — never a deletion mandate. (Origin R1)
- R2. Existing instructions preserved verbatim in intent: distill conclusions into OpenSpec, reference source docs, skip ideation skills when requirements are clear. (Origin R2)
- R3. Both managed-block surfaces carrying "disposable" — `full` tier SSOT paragraph and `orchestrator` tier line — carry the retention reframing. (Origin R3)
- R4. `BLOCK_VERSION` bumps 2 → 3; re-running `init-prj` replaces the block between markers with no migration command. (Origin R4)
- R5. `minimal` tier stays byte-identical; no runtime logic changes outside block text and the version constant. (Origin R5)
- R6. Docs reflecting block contents reflect the retention clarification, including the two concept-repeating guides the user pulled into scope at synthesis. (Origin R6)

**Origin actors:** A1 agents/contributors in this repo, A2 agents/contributors in adopted projects, A3 maintainers deciding when to re-run `init-prj`.
**Origin acceptance examples:** AE1 (retention reading after distillation), AE2 (v2→v3 upgrade on re-run, `block_version == 3`), AE3 (`minimal` byte-identical).

---

## Scope Boundaries

- No doctor checks for adoption-block drift (deferred since v2 brainstorm).
- No changes to skill definitions or FSM stages.
- No automated archival/deletion machinery for ideation artifacts.
- Future process candidates from origin remain captured-only (five items listed in origin Scope Boundaries).

---

## Context & Research

### Relevant Code and Patterns

- `src/commands/init_prj.rs`: `BLOCK_VERSION` (line 20) feeds on-disk header and `state.json`; `render_block_content(tier)` holds `full` SSOT paragraph (~line 89) and `orchestrator` line (~line 107); marker splice + SHA comparison handles re-adoption (lines 183–221); shared classifier `check_adoption_block_status` (34–67) consumed by `doctor.rs:154` and `status.rs:94`.
- `src/harness/custom.rs::render_full_block()`/`ensure_rules_block()` (155–199): second renderer reusing the same constants for custom-harness rules files.
- Precedent change: `openspec/changes/archive/adoption-block-ssot-v2/` and plan `docs/plans/2026-08-22-feat-adoption-block-ssot-v2-plan.md`.

### Institutional Learnings

- `docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md`: bump exactly ONE constant; body SHA recomputes automatically — never hand-edit hashes; v2 itself was a content-only bump upgraded in place.
- `docs/solutions/logic-errors/init-prj-created-file-clobber-on-re-adoption-2026-08-22.md`: version bumps force the re-run/replacement path where `created_file` was historically clobbered; regression test `init_prj_upgrade_rerun_preserves_created_file_flag` must stay green.
- `docs/solutions/architecture/adoption-block-staleness-alignment-across-status-and-doctor.md`: post-bump, every existing v2 block classifies as `StaleVersion` ("re-run ce-ai init-prj") in status/doctor — intended UX.
- `docs/solutions/bugfixes/transactional-error-propagation-and-state-commit.md`: `state.save()` strictly after filesystem mutations; propagate errors with `?`.

External research skipped — strong local precedent (the v2 change did exactly this shape).

---

## Key Technical Decisions

- **Single-constant bump**: changing `BLOCK_VERSION` alone updates header, `state.json.block_version`, and invalidates body SHA; replacement stays marker-based and version-agnostic.
- **Both block tiers reworded**, not only `full`: leaving "disposable" ambiguous in the orchestrator line preserves the hole being closed (origin Key Decision).
- **Custom-harness renderer inherits automatically** via shared constants; only its pinned test needs updating.
- **Extra guides aligned in-scope** (user decision at synthesis): `quick-start-workflow-guide.md` and `fsm-and-checkpoints-explained.md` repeat the concept and would otherwise contradict the clarified rule.
- **Version level**: MINOR bump, matching the verified precedent that the same-shape v2 adoption-block change shipped as 1.5.0 (commit `03c7270`). Rationale: guidance text plus the constant-driven Ok → StaleVersion reclassification of existing adoptions is user-visible, even though runtime logic is untouched. The U4 check against CHANGELOG/git history confirms the heading only — it is not the decision mechanism.
- **Directional wording drafts** (resolves origin's deferred wording question; final polish allowed while editing):

  > Root `AGENTS.md`, appended to the SSOT rule after "…never maintain brainstorm/ideation documents in sync with OpenSpec.":
  > *"Ideation artifacts are retained by default as the permanent raw-history record that OpenSpec deliberately does not duplicate — 'disposable' never means deleting them; removal is an ordinary reversible git decision, never a workflow step."*

  > `full` tier SSOT paragraph, same sentence adapted to the block's terser register (drop the em-dash clause if length matters):
  > *"Ideation artifacts are retained by default as the permanent raw-history record OpenSpec intentionally does not duplicate; 'disposable' never means deleting them."*

  > `orchestrator` tier line:
  > *"Ideation outputs are disposable inputs: distill them into the specs before delegation; never maintain them in parallel; retain them as raw history instead of deleting them."*

---

## Open Questions

### Resolved During Planning

- Guide-alignment scope: user confirmed the two extra guides are in scope (synthesis call-out).
- Wording ambiguity: resolved with directional drafts above (origin deferred-to-planning item).
- Duplicate-wording inventory: research sweep complete — verbatim copies live only in historical artifacts (archived spec, old plan, CHANGELOG v2 entry, solutions architecture doc quoting constants) which stay untouched; concept repeats limited to the three user guides named in R6.

### Deferred to Implementation

- Final sentence-level polish of the drafts while editing each surface.

---

## Implementation Units

### U1. Author OpenSpec change `ideation-retention-ssot-v3`

**Goal:** Satisfy the mandatory OpenSpec gate: distill the origin brainstorm into the formal change contract before any code.

**Requirements:** All (contract covers R1–R6).

**Dependencies:** None.

**Files:**
- Create: `openspec/changes/ideation-retention-ssot-v3/proposal.md`
- Create: `openspec/changes/ideation-retention-ssot-v3/exploration.md`
- Create: `openspec/changes/ideation-retention-ssot-v3/design.md`
- Create: `openspec/changes/ideation-retention-ssot-v3/spec.md`
- Create: `openspec/changes/ideation-retention-ssot-v3/tasks.md`

**Approach:**
- Distill from origin (SSOT rule applies to the brainstorm itself: reference, don't copy).
- `spec.md` uses WHEN/THEN requirements mirroring origin acceptance examples.
- `tasks.md` transcribes U2–U4 below as the executable checklist with per-work-unit changed-line estimates (target ~200 LOC each; all units land well under).

**Patterns to follow:** `openspec/changes/archive/adoption-block-ssot-v2/` file set and tone.

**Test scenarios:**
- Test expectation: none — specification-authoring unit; correctness enforced by review and downstream units.

**Verification:**
- Five files exist under the change folder; every origin requirement maps to at least one WHEN/THEN scenario.

---

### U2. Reword rule surfaces, bump `BLOCK_VERSION` to 3, update pinned tests

**Goal:** Land the retention clarification in all three wording surfaces plus the version bump, keeping the suite green atomically.

**Requirements:** R1, R2, R3, R4, R5.

**Dependencies:** U1.

**Files:**
- Modify: `AGENTS.md`
- Modify: `src/commands/init_prj.rs`
- Modify: `tests/cli.rs`

**Approach:**
- Apply the Key Technical Decisions wording drafts to root `AGENTS.md`, `full` tier paragraph, and `orchestrator` line; keep existing sentences (distill/reference/skip-rules) intact per R2.
- Bump `pub const BLOCK_VERSION: u32 = 3`. Touch nothing else in the render/splice/state path.
- Update literal-version assertions and content-pinning tests:
  - `v=2` header strings → `v=3` in `init_prj_and_deinit_prj_roundtrip_fresh_repo`, `init_prj_preserves_preexisting_content_and_crlf`, `init_prj_replaces_v1_block_with_v2_preserving_content_and_crlf`, `init_prj_replaces_lf_only_v1_block_preserving_content`, `install_custom_with_flags_copies_layout_manifest_state_and_rules_block`.
  - `init_prj_full_tier_contains_ssot_rule`: assert retention sentence present AND skip-sentence still present.
  - `init_prj_orchestrator_tier_contains_distillation_line_once`: update phrase, keep `count() == 1`.
  - `doctor_reports_generic_drift_for_tampered_v2_body`: its injected tamper header hardcodes `v=2`, which post-bump classifies as StaleVersion (2 < 3) — retarget the fixture literal to the current version (`v=3`) with a corrupted SHA so it keeps exercising the generic-drift branch, and rename the test to drop the misleading "v2".
  - `init_prj_minimal_block_matches_v1_bytes`: expect unchanged green (guards R5).
- Add one integration test: v2 fixture adoption → upgrade re-run renders v=3 block, `state.json.block_version == 3`, and `created_file` provenance preserved (extends the existing rerun-preserves-flag test's coverage).

**Patterns to follow:** Existing v1→v2 replacement tests (`init_prj_replaces_v1_block_with_v2_preserving_content_and_crlf`).

**Test scenarios:**
- Covers AE2. Integration — Given an adopted repo whose block is at v=2, when `init-prj --tier full` re-runs after the bump, then the block between markers shows `v=3` with retention wording and `block_version == 3` in state, preserving pre-existing content and the `created_file` flag.
- Covers AE3. Happy path — `minimal` render is byte-identical to the frozen snapshot.
- Happy path — `full` render contains both the new retention sentence and the original skip-skills sentence.
- Happy path — `orchestrator` render contains the updated retention clause exactly once.
- Edge case — CRLF and LF-only legacy blocks each upgrade to v=3 preserving their line-ending style and surrounding bytes.
- Edge case — doctor/status classify an untouched v2 fixture as stale-version with the re-run hint (not generic drift).
- Error path — begin-marker-without-end still fails closed with the file untouched.

**Verification:**
- `cargo test` green including all renamed/updated assertions; `cargo clippy --all-targets --all-features -- -D warnings` clean; grep for `"v=2"` returns hits only inside historical fixtures/docs, never in live templates.

---

### U3. Align user-facing docs

**Goal:** Make every doc that describes the block or restates the SSOT concept consistent with the retention framing.

**Requirements:** R6.

**Dependencies:** U2 (final wording frozen).

**Files:**
- Modify: `docs/user-guide/project-adoption-guide.md` (example header `v=2` → `v=3`; rewrite "(v1 ➔ v2)" upgrade section as "(v2 ➔ v3)" or version-generic; tier rows mention retention)
- Modify: `docs/user-guide/quick-start-workflow-guide.md` (three "disposable input" mentions realigned)
- Modify: `docs/user-guide/fsm-and-checkpoints-explained.md` (one mention realigned)

**Approach:** Minimal edits; keep each guide's voice. Historical artifacts (archived specs, old plans, past CHANGELOG entries) stay untouched.

**Patterns to follow:** How the v2 change updated `project-adoption-guide.md`.

**Test scenarios:**
- Test expectation: none — documentation-only unit.

**Verification:**
- Grep for "disposable" across `docs/` returns only retained-history-consistent usages; adoption guide shows v=3 examples.

---

### U4. Changelog and version release prep

**Goal:** Satisfy the repo DoD: SemVer bump, changelog entry, adoption-impact note.

**Requirements:** R4 (adoption impact communication).

**Dependencies:** U2, U3.

**Files:**
- Modify: `Cargo.toml` (version bump per resolved patch/minor decision)
- Modify: `CHANGELOG.md` (new version heading + entry; do NOT edit the historical v2 entry)

**Approach:**
- Entry describes: SSOT retention clarification in root rules and adoption block v3; explicit note that existing adoptions will surface "stale block version" hints in `status`/`doctor` until `init-prj` is re-run.
- Write the MINOR version heading per Key Technical Decisions (precedent verified: v2 shipped as 1.5.0).

**Test scenarios:**
- Test expectation: none — metadata/documentation unit.

**Verification:**
- `cargo build` succeeds with the bumped version; CHANGELOG follows Keep-a-Changelog format used by previous entries.

---

## System-Wide Impact

- **Interaction graph:** reworded block text fans out to all harness rule-file surfaces (cursor, claude, codex, copilot, grok, kimi, pi, fx, agents, gemini) on re-adoption — intended propagation, no code changes needed there.
- **Error propagation:** unchanged; fail-closed malformed-block behavior and `?`-propagated errors preserved.
- **State lifecycle risks:** none new; writes continue through `write_atomic`, `state.save()` ordering untouched; body SHA recomputes from content.
- **API surface parity:** `custom.rs` renderer shares the constant — covered in U2's pinned-test update.
- **Integration coverage:** upgrade-path test (v2 fixture → v3) proves cross-version splice behavior unit tests can't.
- **Unchanged invariants:** `minimal` tier bytes, marker format, `ProjectAdoptionEntry` schema, staleness classifier semantics (only the constant value shifts classifications from Ok → StaleVersion for un-re-run projects).

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| Missed literal `v=2` or wording assertion breaks CI late | U2 lists every known site; final grep sweep in U2/U3 verification |
| Adopted-project users surprised by stale-block hints | Explicit CHANGELOG/release-note callout (U4) |
| Root `AGENTS.md` vs block wording drifts apart again | Both surfaces derive from the same directional draft in Key Technical Decisions |
| SemVer heading drift vs decided MINOR level | U4 writes the heading directly from Key Technical Decisions (precedent already verified) |

## Documentation / Operational Notes

- Release notes should lead with the adoption impact (staleness hints until re-run), not the wording detail.
- No monitoring/rollout machinery involved.

---

## Sources & References

- **Origin document:** [docs/brainstorms/2026-08-25-ideation-retention-ssot-clarification-requirements.md](docs/brainstorms/2026-08-25-ideation-retention-ssot-clarification-requirements.md)
- Related code: `src/commands/init_prj.rs` (`BLOCK_VERSION`, `render_block_content`, `check_adoption_block_status`), `src/harness/custom.rs` (`render_full_block`)
- Prior art: `openspec/changes/archive/adoption-block-ssot-v2/`, `docs/plans/2026-08-22-feat-adoption-block-ssot-v2-plan.md`
- Learnings: see Context & Research institutional learnings list
