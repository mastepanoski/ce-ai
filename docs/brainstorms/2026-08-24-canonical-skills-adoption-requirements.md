---
date: 2026-08-24
topic: canonical-skills-adoption
---

# Canonical Skills Adoption (Harvest + Registry)

## Summary

ce-ai will harvest the Compound Engineering release's top-level `skills/` tree into a **single canonical surface** with an adaptive destination: harness skill directories that already contain `ce-*` skills are **adopted in place** (rewritten with backup, becoming the managed surface), while fresh machines get the canonical copy in the managed directory. The SkillRegistry indexes the canonical surface and serves skills to any harness on demand; the sync matrix hash-verifies every managed skills surface. No deletions; no new copies in harnesses that had no CE skills of their own.

---

## Problem Frame

ce-ai today propagates only the OpenCode plugin loader: the upstream release ships its 33 skills at top-level `skills/` (not under the harvested `.opencode/skills` prefix), so no harness receives managed skill files and the sync matrix reports them as `registered — nothing to verify` (diagnosed 2026-08-24; wording fix pending in open PR #230 — must merge before this work starts).

Meanwhile, real machines accumulate Compound Engineering skill copies through other channels — manual installs, older tooling, plugin marketplaces. These copies go stale, and harnesses that read both a user skills directory and the managed directory index **both sets**, paying a per-session token tax for duplicated skill descriptions. The affected user is a multi-harness developer who runs `ce-ai sync` expecting verified, current CE assets everywhere and instead gets unverified surfaces plus silent duplication.

The token cost lives in skill-description indexing per harness session, not in disk copies — so the design goal is **one indexed set per harness, always fresh**, not "copy everywhere".

---

## Actors

- A1. **Multi-harness developer**: runs `ce-ai install`/`sync`; owns the user-level skill directories; pays the token cost.
- A2. **ce-ai**: harvests, adopts, verifies, and indexes; must never destroy user-authored content (repo hard-gate #4).
- A3. **Harnesses** (opencode, pi, claude, codex, …): consume skills from their own conventions; some hold pre-existing `ce-*` copies, some hold none.

---

## Key Flows

- F1. **First adoption via the explicit adopt command**
  - **Trigger:** sync or install detects a harness skills directory already containing `ce-*` skill folders and reports it as `pending-adoption`; the user then runs the explicit adopt command.
  - **Actors:** A1, A2
  - **Steps:** adopt command lists the adoptable surface (paths, stale vs current) → asks for explicit confirmation → stages the canonical content → backs up each file it will rewrite → rewrites `ce-*` skills (completing the set) → writes the surface manifest atomically → records the surface as managed; any failure auto-restores and leaves the surface unadopted.
  - **Outcome:** the pre-existing location becomes the managed, matrix-verified surface; no duplicate copy is created in the managed directory; nothing outside `ce-*` names was touched.
  - **Covered by:** R2, R3, R7, R9, R10, R15, R17
- F2. **On-demand resolution for harnesses without local copies**
  - **Trigger:** a harness with no managed skills (e.g. codex) needs a CE skill.
  - **Actors:** A1, A2, A3
  - **Steps:** user (or harness workflow) runs the skills resolver → registry matches the query against the canonical index → returns verified canonical paths for that harness.
  - **Outcome:** the harness uses canonical skills without ce-ai creating local copies.
  - **Covered by:** R6

---

## Requirements

**Harvest and canonical destination**
- R1. `install`/`sync` must harvest the release's top-level `skills/` tree as the canonical CE skill source.
- R2. Destination is adaptive: if a harness skills directory already contains `ce-*` skill folders, ce-ai adopts that location in place (rewrite to canonical version) and treats it as the managed surface for that harness. Only when no harness skills directory on the machine contains `ce-*` skill folders does ce-ai write the canonical copy — exclusively to the OpenCode managed directory; no other harness receives skill files. Adoption completes an adopted surface to the full canonical set: missing `ce-*` skills are written alongside the rewritten ones, and the matrix reports verification against the canonical skill count.
- R3. The first adoption of each surface requires explicit user confirmation; after adoption, routine `sync` rewrites of that surface are automatic. If the user declines, the surface stays unmanaged and untouched, the matrix reports it as `registered` with the canonical-copy guidance note, and ce-ai does not re-prompt until an explicit adopt command is run.
- R4. ce-ai must never write skill files into harness-owned skills directories of harnesses that had no pre-existing `ce-*` skills (token-neutrality); such harnesses keep their `registered` matrix status.
- R12. Marketplace/plugin-channel-origin `ce-*` copies are detected and reported as unmanaged duplicates but are never adoptable.
- R13. When adoption designates a new managed surface for a harness, ce-ai retires its previously managed copy for that same harness (removing only manifest-tracked files ce-ai authored, with backup), so at most one managed skills surface per harness exists at any time.
- R14. Adoption confirms the candidate directory matches the harness's current skills-root convention; when it cannot, ce-ai warns and skips adoption rather than recording an unreadable managed surface.

**Registry and resolution**
- R5. The SkillRegistry must index the canonical/adopted surface wherever it lives, refreshed after each sync.
- R6. Skills resolution must serve canonical CE skills to any harness on demand, falling back to the canonical path when the harness has no local copy.

**Verification and matrix**
- R7. The sync verification matrix must hash-verify every managed skills surface (adopted or canonical), reporting `verified N/N` per surface.
- R8. Harnesses without managed skills keep the `registered` wording, with the guidance note pointing to the canonical copy and on-demand resolution.

**Safety**
- R9. Rewrites are restricted to `ce-*`-named skill directories; user-authored skills are never touched.
- R10. Every rewritten file is backed up before writing and restorable.
- R15. Adoption is transactional: changes stage before applying, the surface manifest is written atomically last, and on any mid-adoption failure ce-ai auto-restores from the just-written backups, leaves the surface unadopted, and exits non-zero.
- R16. After adoption, manual edits to managed `ce-*` files are treated as drift: the next sync backs up and restores them, reporting each as `restored-drift` in the matrix.
- R17. Non-interactive contexts (no TTY, TUI-spawned runs, `--dry-run`, `--quiet`) never adopt: adoptable surfaces report as `pending-adoption` in the matrix, and adoption happens only through the explicit adopt command, which confirms per surface (bypassable with `--yes`).
- R18. Marketplace-origin duplicates get a distinct `external-duplicate` matrix state naming the offending paths with manual-removal guidance; resolution excludes them.
- R19. `status` and `doctor` surface adoption states (adopted / pending-adoption / declined / external-duplicate); a vanished adopted root is reported as orphaned and requires explicit re-adoption.

**Documentation**
- R11. User documentation must explain the canonical/adoption model, the first-adoption confirmation prompt, and on-demand resolution.

---

## Acceptance Examples

- AE1. **Covers R2, R3, R7, R10.** Given a machine where `~/.config/opencode/skills` holds stale `ce-*` folders, when the first post-upgrade `ce-ai sync` runs, ce-ai reports the adoptable surface and asks; on confirmation it backs up, rewrites the `ce-*` skills to canonical, creates no managed-directory copy, and the matrix shows `opencode: verified 33/33`.
- AE2. **Covers R4, R8.** Given a fresh machine where claude is detected but has no `ce-*` skills, when sync runs, ce-ai registers MCP companions only, writes zero skill files into harness-owned directories (the canonical copy lands in the OpenCode managed directory per R2), and the matrix shows claude as `registered` with the canonical-copy guidance note.
- AE3. **Covers R9.** Given a user skills directory containing `ce-debug` (stale) and `my-own-skill` (user-authored), when adoption runs, `ce-debug` is rewritten and `my-own-skill` is untouched.
- AE4. **Covers R6.** Given codex with no local CE skills, when the user runs skills resolution for a task, the output injects canonical, SHA256-verified skill paths.
- AE5. **Covers R1.** Given a resolved CE release whose tree contains top-level `skills/`, when install or sync runs, the harvest stages that tree as the canonical CE skill source.
- AE6. **Covers R5.** Given pi's surface adopted at its real skills root, after sync the registry resolves pi skills from that adopted path — not from `~/.ce-ai/harness-pi/skills`.

---

## Success Criteria

- On the reference machine (opencode + pi with stale copies, claude via marketplace), each ce-ai-managed harness (opencode, pi) indexes exactly one fresh, hash-verified copy of each CE skill; marketplace-managed harnesses (claude) keep their external copy — reported as external with staleness guidance, never modified. No duplicated descriptions, no deletions.
- A harness without pre-existing CE skills receives none; its token footprint is unchanged.
- Duplicated skill-description token overhead is measured on the reference machine before implementation and re-measured after; the adopted design demonstrably reduces it to zero for managed harnesses.
- The matrix verifies every managed skills surface, and the user guide explains the canonical/adoption model without external help.

---

## Scope Boundaries

- No symlink farms and no per-harness copy creation (rejected direction B).
- No per-harness subset selection (rejected direction C).
- No management of marketplace/plugin-channel installs (e.g. Claude Code plugins) — at most reported, never modified.
- No deletion of user files: adoption rewrites, never removes — except ce-ai-authored, manifest-tracked files during managed-surface retirement (R13); non-`ce-*` content untouched.
- Other release assets (`.claude/commands`, `.agents/plugins`, etc.) are not harvested.

---

## Key Decisions

- **Canonical copy + registry over per-harness copies**: the token cost is per-harness description indexing, so the goal is one indexed set per harness, not copies everywhere (user driver: token economy).
- **Adopt-in-place over prune**: preference-based choice, not technically forced — both yield one indexed set per harness. Adopt-in-place honors "no deletions" and preserves user-chosen locations, at the cost of managed surfaces scattered across user directories (manifest path recording, per-surface matrix logic, uninstall coherence); confirm-and-prune with backup would centralize managed surfaces but delete user-placed copies.
- **Uniform adoption rule across harnesses**: any harness with pre-existing locally-installed `ce-*` copies is adoptable (user confirmed; on the reference machine this covers opencode and pi). Marketplace/plugin-channel-origin copies are detected and reported as unmanaged duplicates but are never adoptable (see R12).
- **Opt-in first adoption**: repo hard-gate #4 (never overwrite user configurations) requires explicit confirmation before ce-ai first writes into user-owned skill directories.

---

## Dependencies / Assumptions

- The upstream release keeps shipping top-level `skills/` as the canonical skill source (verified for v3.23.3).
- The `registered` matrix wording and guidance note (R8) ship in PR #230, which must merge into `main` before this work starts.
- Per-harness skills-root conventions (`config/skills` nesting for agy, etc.) remain stable.
- Verified gap to fix during planning: the registry's harness-root scan points at `~/.ce-ai/harness-<kind>/skills`, not the real harness directories where adopted surfaces live — scan roots must be aligned (R5).

---

## Outstanding Questions

### Resolve Before Planning

- (none)

### Deferred to Planning

- [Affects R2][Technical] Detection heuristic for an adoptable `ce-*` skill folder (directory-name prefix vs SKILL.md frontmatter name) and stale/current classification.
- [Affects R2, R7][Technical] How the install manifest tracks adopted surfaces that live outside the managed directory (managed-relative vs recorded absolute paths) while keeping uninstall coherent.
- [Affects R6][Technical] Registry `resolve` fallback mechanics for harnesses without local copies — evaluate first the native additional-skills-root registration pattern (precedent: `merge_skills_path` appending managed paths into harness config in `src/opencode/config.rs`), falling back to registry resolve only for harnesses without such support.
