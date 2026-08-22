---
title: "feat: Adoption Block SSOT Guidance (v2)"
type: feat
status: active
date: 2026-08-22
origin: docs/brainstorms/2026-08-22-openspec-ssot-adoption-block-requirements.md
---

# feat: Adoption Block SSOT Guidance (v2)

## Summary

Carry the Single Source of Truth / anti-duplication guidance into the managed
AGENTS.md blocks that `ce-ai init-prj` injects into adopted projects: a full
section in the `full` tier, one distillation directive line in the
`orchestrator` tier, `minimal` pinned byte-identical, and the managed-block
header/state version bumped to a shared `BLOCK_VERSION = 2` constant.
Implementation follows the contract at
`openspec/changes/adoption-block-ssot-v2/`; this plan sequences those same
units by ID and maps files/tests — content lives there, not here (DRY).

---

## Requirements

- R1. Rendered `full` block contains the SSOT rule + KISS skip rule; rendered
  `orchestrator` block contains exactly one added distillation line;
  `minimal` output stays byte-identical to v1. (Origin Success Criteria;
  contract Scenarios 1-3.)
- R2. Header `v=` field and `ProjectAdoptionEntry.block_version` both derive
  from one `BLOCK_VERSION` constant equal to 2. (Contract Scenario 4.)
- R3. Re-running `init-prj` on a project adopted with a v1 block replaces only
  the text between markers, preserves surrounding user content and CRLF, and
  records `block_version == 2`. (Contract Scenario 5.)
- R4. A consecutive identical run reports up-to-date and leaves `AGENTS.md`
  bytes untouched. (Contract Scenario 6.)

---

## Scope Boundaries

- No changes to skill definition files, CLI flags/commands (`migrate-prj`
  explicitly rejected), or `deinit-prj` restore semantics.
- No doctor checks for adoption-block drift (deferred consideration).
- Marker format (`<!-- ce-ai:block begin/end -->`) unchanged.

### Deferred to Follow-Up Work

- Doctor-based drift detection for stale v1 blocks in adopted projects:
  future iteration if operators report stale-block confusion.

---

## Context & Research

### Relevant Code and Patterns

- `src/commands/init_prj.rs` — `render_block_content(tier)` static strings;
  header built inline in `run()` with literal `v=1`;
  `ProjectAdoptionEntry { block_version: 1, .. }`; sha256 comparison via
  whole-rendered-block equality drives idempotent replacement.
- `src/state/state.rs` — `AdoptionTier`, `ProjectAdoptionEntry`.
- Two tests in `tests/cli.rs` pin the `v=1` header and must be updated to
  `v=2`: `init_prj_and_deinit_prj_roundtrip_fresh_repo` (pins
  `v=1 tier=full`) and `init_prj_preserves_preexisting_content_and_crlf`
  (pins `v=1 tier=minimal`; also the CRLF preservation pattern to mirror).
- Verified against source this session (CodeGraph read + two review agents
  cross-checked every factual claim).

### Institutional Learnings

- `docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md`
  — documents the marker/block lifecycle; keep it accurate if behavior notes
  change (they do not: markers and replacement logic are untouched).

---

## Key Technical Decisions

- Introduce `pub const BLOCK_VERSION: u32 = 2` used by both the header
  `format!` and the state entry — eliminates the two-independent-literals
  drift found during exploration (contract design §1).
- Plan units reuse the contract's U-IDs verbatim (U1-U9): the plan sequences
  and maps them; authoritative content stays in
  `openspec/changes/adoption-block-ssot-v2/{design,tasks,spec}.md` (project
  SSOT convention).
- Test scenarios trace to contract scenarios S1-S6 using the `Covers S<N>.`
  prefix convention.

---

## Open Questions

### Resolved During Planning

- Unit granularity and ordering: taken 1:1 from contract `tasks.md` (no
  re-plan needed; contract was authored with TDD ordering already).

### Deferred to Implementation

- Exact assertion strings inside the two updated `tests/cli.rs` tests:
  depend on final rendered text; mechanical once U2/U3 land.

---

## Implementation Units

Unit content is specified in `openspec/changes/adoption-block-ssot-v2/tasks.md`
and `design.md`; entries below add sequencing, dependency, and trace mapping
only. All units modify within the worktree repo root.

### U1. Shared `BLOCK_VERSION` constant

**Goal:** Single source for header `v=` and state `block_version`.
**Requirements:** R2 · **Dependencies:** None
**Files:** Modify `src/commands/init_prj.rs`; Test `tests/cli.rs`
**Approach:** Contract task U1 (Red: flip both existing pinned assertions to
expect `v=2`). Covers S4.
**Verification:** Existing test passes with `v=2` in header.

### U2. SSOT section in `full` tier

**Goal:** Full-tier block carries SSOT + KISS-skip guidance.
**Requirements:** R1 · **Dependencies:** None (parallel-safe with U1)
**Files:** Modify `src/commands/init_prj.rs`; Test `tests/cli.rs`
**Approach:** Contract task U2; verbatim text in contract `design.md` §2.
Covers S1.
**Verification:** New test asserts both rule strings present.

### U3. Distillation line in `orchestrator` tier

**Goal:** One-line disposable-input directive.
**Requirements:** R1 · **Dependencies:** None
**Files:** Modify `src/commands/init_prj.rs`; Test `tests/cli.rs`
**Approach:** Contract task U3; verbatim text in contract `design.md` §3.
Covers S2.
**Verification:** Assertion finds the line exactly once.

### U4. `minimal` byte-equality guard

**Goal:** Pin minimal output to v1 bytes forever.
**Requirements:** R1 · **Dependencies:** None
**Files:** Test `tests/cli.rs`
**Approach:** Contract task U4. Covers S3.
**Verification:** Byte-equality test green.

### U5. v1 → v2 in-place replacement integration test

**Goal:** Prove upgrade path on already-adopted projects.
**Requirements:** R3 · **Dependencies:** U1, U2, U3
**Files:** Test `tests/cli.rs`
**Approach:** Contract task U5 (hand-written v1 block + CRLF + user text).
Covers S5.
**Verification:** Markers-only replacement asserted; state has
`block_version == 2`.

### U6. Idempotent second run

**Goal:** Lock double-run no-op behavior.
**Requirements:** R4 · **Dependencies:** U5
**Files:** Test `tests/cli.rs`
**Approach:** Contract task U6. Covers S6.
**Verification:** Second run reports up-to-date; file bytes unchanged.

### U7. Adoption guide documentation

**Goal:** Users discover the v2 contents and the re-run upgrade path.
**Requirements:** Origin Success Criteria bullet 3 · **Dependencies:** U2, U3
**Files:** Modify `docs/user-guide/project-adoption-guide.md`
**Approach:** Contract task U7.
**Test expectation:** none — documentation only.

### U8. Ship preparation

**Goal:** SemVer + changelog ready for release flow.
**Requirements:** Repo DoD · **Dependencies:** U1-U7
**Files:** Modify `Cargo.toml`, `Formula/ce-ai.rb`, `CHANGELOG.md`
**Approach:** Contract task U8; MINOR bump (new feature).
**Test expectation:** none — metadata only.

### U9. Full verification gate

**Goal:** DoD compliance before PR.
**Requirements:** Repo DoD · **Dependencies:** U8
**Files:** None (execution gate)
**Approach:** Contract task U9: fmt, clippy `-D warnings`, `cargo test`,
`make e2e`.
**Verification:** All four gates green.

---

## System-Wide Impact

- **Unchanged invariants:** marker format, `deinit-prj` restore logic,
  `write_atomic` write path, `state.json` schema (field types unchanged),
  TUI adoption flows (`run_init_prj_cmd` delegates to the same `run()`).
- **Integration coverage:** U5 exercises the full harness→file→state loop
  that unit tests alone cannot prove.

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| Adopted projects stay on v1 blocks until operators re-run | Accepted; documented in U7 (silent remote mutation would violate preserve-user-configs invariant) |
| Other in-flight branches touch `init_prj.rs` concurrently | Worktree isolation (`feat/openspec-ssot`); rebase before PR |

---

## Sources & References

- **Origin document:** docs/brainstorms/2026-08-22-openspec-ssot-adoption-block-requirements.md
- **Contract:** openspec/changes/adoption-block-ssot-v2/ (proposal,
  exploration, design, spec, tasks)
- Related code: src/commands/init_prj.rs, src/state/state.rs, tests/cli.rs
- Institutional learning: docs/solutions/architecture/project-adoption-engine-init-and-deinit-prj.md
