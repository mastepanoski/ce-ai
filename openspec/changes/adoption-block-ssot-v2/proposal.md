# Proposal: Adoption Block SSOT Guidance (v2)

## Problem Statement

Agents working in projects adopted via `ce-ai init-prj` receive a managed
`AGENTS.md` block that mandates the 7-stage cycle and OpenSpec but gives no
rule about how ideation artifacts relate to OpenSpec. Result: agents maintain
`docs/brainstorms/` / `docs/ideation/` as parallel specifications, duplicating
content and burning tokens. The ce-ai repository already codified the fix in
its own root `AGENTS.md` ("Single Source of Truth Rule"); the injected template
must carry the same guidance so adopted projects inherit it.

Source requirements: `docs/brainstorms/2026-08-22-openspec-ssot-adoption-block-requirements.md`
(disposable input — decisions distilled here).

## In Scope

- `full` tier block: add Single Source of Truth rule + KISS skip rule.
- `orchestrator` tier block: add exactly one line stating ideation outputs are
  disposable inputs for OpenSpec, not parallel specs.
- Managed block header version bump `v=1` ➔ `v=2`; `state.json`
  (`ProjectAdoptionEntry.block_version`) records the new version.
- Integration tests covering new content, minimal-tier regression guard, and
  v1 ➔ v2 block replacement on re-run of `init-prj`.
- Update `docs/user-guide/project-adoption-guide.md`.

## Out of Scope

- Skill definition changes; new CLI commands or flags; doctor drift checks for
  adoption blocks; any change to `deinit-prj` restore semantics.

## Risk Evaluation

- **Low**: template is a static string change; marker format unchanged, so
  deinit and idempotent re-run logic are untouched.
- **Mitigated**: `minimal` tier pinned by byte-equality regression test.
- Users with v1 blocks get the new guidance only after re-running `init-prj`;
  documented in the adoption guide (acceptance, not a blocker).

## Success Criteria

1. `cargo test` green including new assertions.
2. Re-running `init-prj` on a project adopted with a v1 block replaces the
   managed block and sets `block_version = 2`, preserving user content around
   the block and CRLF line endings.
3. Adoption guide documents the new block contents and the re-run path.
