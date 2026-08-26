# Proposal: canonical-skills-adoption

## Why

The compound-engineering release ships its 33 skills at top-level `skills/`, which ce-ai never harvested — no harness receives managed skill files, the sync matrix reports every surface as `registered — nothing to verify` (diagnosed 2026-08-24), and real machines accumulate stale CE copies through manual installs and marketplaces while paying a per-session token tax for duplicated skill descriptions. Requirements and review history: `docs/brainstorms/2026-08-24-canonical-skills-adoption-requirements.md` (R1-R19, AE1-AE6); implementation plan: `docs/plans/2026-08-24-001-feat-canonical-skills-adoption-plan.md`.

## What Changes

- Harvest the release's top-level `skills/` tree (extend `MANAGED_PREFIXES`); fresh machines get the canonical copy in the OpenCode managed directory; `copy_managed_skills` call sites removed so harness-owned directories never receive unrequested copies (R1, R2, R4).
- New `skills adopt` command with an adoption ledger (`skill_surfaces` in state.json): detects adoptable `ce-*` surfaces (frontmatter-verified, path-validity checked), transactional rewrite with per-file backup, journal protection, partial-set completion, and managed-surface retirement (R3, R9, R13, R14, R15).
- Sync keeps adopted surfaces current (rewrites + `restored-drift` reporting) and the matrix hash-verifies every managed surface with new states: `pending-adoption`, `external-duplicate`, `orphaned` (R7, R8, R12, R16, R17).
- Uninstall becomes ledger-scoped on harness skills directories (never `remove_dir_all` where user content may live), cleaning ledger entries with the removal (R9, R13 parity).
- SkillRegistry indexes ledger-tracked surfaces + managed dir for all harnesses (drops the dead Tier-3 root); `status`/`doctor` surface adoption states (R5, R6, R19).
- User guide documents the canonical/adoption model (R11); SemVer minor bump + CHANGELOG.

## Scope

- Non-goals (from origin): symlink farms, per-harness subset selection, marketplace/plugin-channel management (detected and reported only), harvesting non-skills release assets.
- Delivery: four chained PRs (~400 lines each) per CONTRIBUTING §4 — see plan Phased Delivery. PR #230 (merged, #13cb138) is a completed dependency.

## Risks

- Uninstall rework regressions on legacy states — mitigated by updated per-harness tests + preservation scenarios (U6).
- Upstream release layout drift — harvest no-ops with warning; surfaces keep last-known content.
- External-origin (marketplace) detection brittleness — v1 scopes to known plugin-cache roots; refinement deferred.

## Success Criteria

- Origin success criteria hold: one fresh hash-verified copy per managed harness, zero new copies for skill-less harnesses, matrix verifies every managed surface, guide self-serves; token-overhead baseline captured before Phase 1 and re-measured after Phase 4.
- All AGENTS.md gates green: fmt, clippy `-D warnings`, cargo test, `make e2e` (container fixture extended), 100% green CI matrix.
