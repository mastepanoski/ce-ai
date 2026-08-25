# Requirements: Ideation Artifact Retention (SSOT Clarification)

Date: 2026-08-25
Status: Approved via ce-brainstorm (single-question flow; scope closed by user)

## Summary

Clarify the Single Source of Truth rule so that "disposable inputs" explicitly means
*retain without syncing* — never a deletion mandate — positioning ideation artifacts
(`docs/brainstorms/`, `docs/ideation/`) as the raw-history layer OpenSpec deliberately
does not duplicate. The clarification lands in the root `AGENTS.md` AND in the managed
adoption block (block bump v=2 -> v=3) so adopted projects inherit the reframing.

---

## Problem Frame

OpenSpec intentionally distills conclusions into `proposal.md` / `exploration.md` /
`design.md` and *references* the source ideation doc instead of copying content
("Single Source of Truth Rule", root `AGENTS.md`; rendered into adopted projects by
the managed block in `src/commands/init_prj.rs`). Multiple archived changes carry live
references to these files (e.g., `openspec/changes/archive/tui_workflow_stage_exec/proposal.md`,
and several adapter `tasks.md` checklists). Because the rule calls ideation artifacts
"disposable inputs" without defining disposal, agents and contributors can reasonably
read it as authorization to delete them once distilled — losing raw context (rejected
alternatives, full requirement reasoning) that lives nowhere else, and breaking those
references. The ambiguity exists in three surfaces today: root `AGENTS.md`, the `full`
tier SSOT paragraph of the managed block (`src/commands/init_prj.rs:89`), and the
orchestrator-tier line (`src/commands/init_prj.rs:107`).

---

## Actors

- A1. Agents/contributors in this repo: read the SSOT rule and decide whether ideation docs may be deleted or must be kept.
- A2. Agents/contributors in adopted projects: read the injected managed block carrying the same rule.
- A3. Maintainers of already-adopted projects: decide when to re-run `init-prj` to pick up the updated block.

---

## Requirements

**SSOT rule wording**
- R1. The SSOT rule must state explicitly that ideation artifacts are retained by default as the permanent raw-history layer, and that "disposable" means *not maintained in sync with OpenSpec and never treated as specifications* — never a mandate to delete.
- R2. The clarified rule must preserve the existing instructions unchanged: distill conclusions into OpenSpec files, reference the source doc instead of copying content, and skip ideation skills when requirements and approach are already clear.

**Managed adoption block propagation**
- R3. Both surfaces of the managed block carrying the word "disposable" — the `full` tier SSOT paragraph and the `orchestrator` tier line — must reflect the same retention reframing.
- R4. The managed block version bumps v=2 -> v=3 following the established pattern; re-running `init-prj` on an already-adopted project replaces the block between markers with no migration command.
- R5. Behavior outside the block content stays untouched: `minimal` tier output remains byte-identical, and no runtime logic changes.

**Documentation consistency**
- R6. User-facing docs that describe the block contents (e.g., `docs/user-guide/project-adoption-guide.md`) must reflect the retention clarification.

---

## Acceptance Examples

- AE1. **Covers R1, R2.** Given an agent in this repo that just distilled a brainstorm into an OpenSpec change, when it reads the SSOT rule considering cleanup, then it understands retention is the default and deletion is neither prescribed nor forbidden (an ordinary reversible git decision backed by history).
- AE2. **Covers R3, R4.** Given an adopted project whose block is at v=2, when the maintainer upgrades the binary and re-runs `ce-ai init-prj <project> --tier full`, then the block between markers is replaced with v=3 including the retention clarification, and `state.json` records `block_version = 3`.
- AE3. **Covers R5.** Given an adopted project on tier `minimal`, when any `init-prj` run completes, then the rendered output is byte-identical to the v2-era `minimal` output.

---

## Success Criteria

- Neither a human nor an agent can reasonably interpret "disposable inputs" as authorization to delete ideation artifacts, in any of the three surfaces (root `AGENTS.md`, `full` tier paragraph, `orchestrator` tier line).
- Integration tests pinning the block header pass at v=3; new assertions cover the retention wording in `full` and `orchestrator` tiers.
- All archived-spec references to `docs/brainstorms/*` continue to resolve against files on disk (nothing is deleted as part of this change).
- Zero ongoing maintenance cost added: no new checks, scripts, sync steps, or CI jobs.

---

## Scope Boundaries

- Process-improvement candidates captured below stay documented only — no changes opened in this cycle.
- No doctor checks for adoption-block drift (already deferred in the v2 brainstorm).
- No changes to skill definitions (`ce-brainstorm`, `ce-ideate`, ...) or the workflow FSM stages.
- No automated archival/deletion machinery for ideation artifacts.

### Future process candidates (captured, not committed)

From an external PM critique reviewed during this brainstorm; each would need its own OpenSpec change if pursued:

1. Consolidate per-change artifacts from five files toward three (proposal/spec/tasks), keeping exploration+design for heavy work only.
2. Expected-outcome fields in proposals plus post-ship outcome verification.
3. Formal human judgment gates (scope / spec acceptance / ship).
4. WIP limits and staleness cadence for open changes.
5. Value-progress surfacing in TUI/status output.

---

## Key Decisions

- **Retention by default instead of a distill-then-delete protocol**: retaining costs ~nothing (small git-tracked markdown read only when referenced), while safe-deletion machinery costs forever; git history already makes accidental deletion recoverable.
- **Propagate to the managed block (user decision)**: adopted projects inherit the reframing; uptake happens on `init-prj` re-run, matching the v2 no-migration precedent.
- **Adjust both block surfaces, not only the `full` tier**: leaving "disposable" ambiguous in the orchestrator line preserves exactly the hole this change closes.
- **Narrow scope with internal candidate list (user decision)**: absorbing the full process critique now would recreate the over-process risk the critique itself warns about.

---

## Dependencies / Assumptions

- Relies on the marker-based block replacement + SHA comparison already shipped and verified in the v2 change.
- Assumes the three verified surfaces (root `AGENTS.md`, `src/commands/init_prj.rs:89`, `src/commands/init_prj.rs:107`) are the only places carrying the ambiguous wording; a full-text sweep is a cheap planning-time verification step.

---

## Outstanding Questions

### Deferred to Planning

- [Affects R1, R3][Technical] Final sentence-level wording and its placement within each surface (English, consistent with existing rule text).
- [Affects R6][Technical] Complete inventory of docs quoting the old rule wording (grep sweep during planning).
