---
date: 2026-08-24
topic: pr-size-policy-alignment
focus: alignment with Compound Engineering workflow; repo-only vs product surface
mode: repo-grounded
---

# Ideation: PR Size Policy — CE Workflow Alignment & Product Surface

## Grounding Context

Codebase context: ce-ai installs Compound Engineering governance into target repos (init-prj governance blocks, hooks, exit-code conventions, doctor probes). The CE workflow is the 7-stage flywheel; Stage 3 (`ce-plan`) generates `tasks.md` with ~200-line work-unit budgets (adopted in #108/v1.22.1); Stage 5 mandates empirical verification before shipping. Gentle AI upstream enforces its Review Workload Guard inside `internal/assets/hermes/sdd-orchestrator.md` after `sdd-tasks`, reading exactly that forecast, with delivery strategies (`ask-on-risk`/`auto-chain`/`single-pr`/`exception-ok`) and chain strategies (`stacked-to-main`/`feature-branch-chain`). The policy itself landed in CONTRIBUTING.md via #226 (repo-only today).

## Topic Axes

- Where the policy lives (this repo's CONTRIBUTING / installable governance block / shared CE template)
- Enforcement point (CI numstat job / Stage-3 forecast / Stage 5→4 bounded correction)
- Audience (ce-ai developers only / any repository adopting CE)

## Ranked Ideas

### 1. Normative estimation convention (ADOPTED in v1.22.1)
**Description:** `tasks.md` work units MUST carry per-work-unit changed-line estimates (~200 target) so the PR forecast is derivable by summing them; codified normatively in AGENTS.md Stage-2 checklist + CONTRIBUTING §4. Enforcement remains manual (PR-template forecast) until the CI numstat job lands.
**Axis:** Enforcement point
**Basis:** direct: CE has NO forecast producer today — `ce-plan` emits no LOC estimates (verified against installed SKILL.md and every active tasks.md); the only upstream mechanism (gentle-ai Review Workload Guard reading sdd-tasks output) does not exist in Compound Engineering.
**Rationale:** Without a producer, the boundary can only be enforced after the fact; the convention makes future forecasts derivable while the CI job closes the loop.
**Downsides:** Relies on authoring discipline until automated.
**Confidence:** 85%
**Complexity:** Low
**Status:** Explored

### 2. Fail-closed CI numstat job
**Description:** `pr-size-budget` CI job failing PRs over 400 lines without a `size:exception` label; closes the fast-follow promised in #108-R1.
**Axis:** Enforcement point
**Basis:** direct: #108-R1 + the fail-closed gate philosophy established in v1.21.4.
**Rationale:** Turns documented policy into enforced policy with zero product scope creep.
**Downsides:** Label workflow needs maintainer discipline.
**Confidence:** 85%
**Complexity:** Low-Medium
**Status:** Unexplored

### 3. Installable size-policy (opt-in)
**Description:** Extend `init-prj` (opt-in tier/flag) to optionally install a size-policy section into target repos' AGENTS.md plus a template numstat CI job.
**Axis:** Where the policy lives / Audience
**Basis:** external: init-prj already installs governance blocks and hooks; gentle-ai embeds the guard in its orchestrator prompt — the analogous move for ce-ai.
**Rationale:** Makes the policy serve every repo adopting CE, not just this one.
**Downsides:** Imposed opinion/friction risk (issue R6); requires product decision via brainstorm.
**Confidence:** 60%
**Complexity:** Medium
**Status:** Unexplored

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| 1 | Drop the limit; mandatory forecast only | Contradicts #108-R1 (high risk of sprawling PRs returning silently) |
| 2 | Size-policy installed by DEFAULT in every target repo | Issue R6 friction/imposed-opinion risk; must be opt-in |
| 3 | Doctor probe auditing host CONTRIBUTING sections | Out of scope: doctor audits CE installations, not host contribution policies |
| 4 | Duplicate variants of S1/S2/S3 across frames | Merged into survivors |

## Correction Log

- Initial survivor 1 claimed the forecast "already originates from the workflow" by analogy with gentle-ai's sdd-tasks → Review Workload Guard chain. **Wrong**: Compound Engineering uses `ce-plan` (no sdd-tasks/guard), and no active tasks.md carries estimates. Corrected to a normative authoring convention plus post-hoc enforcement after maintainer challenge.

## Open Question for Brainstorm

If S3 is pursued: what does `init-prj` install by default vs opt-in, and does the numstat job belong to ce-ai's own CI or ship as a template?
