# Requirements: SSOT Guidance in Adoption Blocks (v2)

Date: 2026-08-22
Status: Refined via ce-brainstorm (single-question flow; scope closed by user)

## Problem

The managed block `ce-ai init-prj` injects into adopted projects' `AGENTS.md`
(`render_block_content`, `src/commands/init_prj.rs`) lists the 7 stages and the
5 OpenSpec files but carries no anti-duplication guidance. Agents working in
adopted projects therefore treat `docs/brainstorms/*.md` and `docs/ideation/*.md`
as parallel specifications and burn tokens keeping them "in sync" with OpenSpec.
The ce-ai repo itself already fixed this gap in its root `AGENTS.md`
(Single Source of Truth Rule); the injected template lags behind.

## Decisions (closed)

1. **Tier scope**: the SSOT + KISS-skip guidance goes into the `full` tier
   (full section) and as a single line in the `orchestrator` tier (it already
   instructs using `ce-brainstorm`, so it is the one other place where parallel
   source confusion can start). `minimal` stays minimal by design (KISS).
2. **Block version**: managed block header bumps `v=1` ➔ `v=2`. Re-running
   `init-prj` on already-adopted projects replaces the block between markers
   (sha comparison already handles this); `state.json` records the new
   `block_version`/`block_sha256`.
3. **Re-adoption path**: no migration command needed — users update by running
   `ce-ai init-prj <project> --tier <t>` again after upgrading the binary.

## Out of Scope

- Changes to skill definition files (`ce-brainstorm`, `ce-ideate`, ...).
- New CLI flags or commands (no `migrate-prj`; re-running `init-prj` suffices).
- Doctor checks for adoption-block drift (future consideration).
- `deinit-prj` behavior (marker-based restore is version-agnostic).

## Success Criteria

- Rendered `full` block contains the Single Source of Truth rule and the
  KISS skip rule; `orchestrator` contains exactly one added line; `minimal`
  output is byte-identical to v1.
- Existing integration tests pass after updating the two assertions in
  `tests/cli.rs` that pin the block header to `v=1` (they must assert `v=2`);
  new tests assert the new content and that
  re-running `init-prj` on a v1-adopted project replaces the block and bumps
  `block_version` to 2 in `state.json`.
- `docs/user-guide/project-adoption-guide.md` reflects the new block contents.
