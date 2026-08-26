---
title: "feat: Canonical skills adoption (harvest + adopt-in-place + registry)"
type: feat
status: active
date: 2026-08-24
origin: docs/brainstorms/2026-08-24-canonical-skills-adoption-requirements.md
---

# feat: Canonical Skills Adoption (Harvest + Adopt-in-Place + Registry)

## Summary

ce-ai harvests the release's top-level `skills/` tree as the canonical CE skill source, adopts pre-existing `ce-*` skill directories in harness skills roots in place (explicit `skills adopt` command, transactional rewrite with backup), keeps the OpenCode managed directory as the canonical surface for fresh machines, extends the SkillRegistry to index and resolve managed surfaces for any harness, hash-verifies every managed skills surface in the sync matrix (with `pending-adoption`, `external-duplicate`, `restored-drift`, and `orphaned` states), and scopes uninstall to manifest/ledger-tracked files so user-authored skills are never destroyed.

---

## Problem Frame

The upstream compound-engineering release ships its 33 skills at top-level `skills/`, which ce-ai never harvested — so no harness receives managed skill files and the sync matrix reports them as `registered — nothing to verify` (diagnosed 2026-08-24; wording fix pending in PR #230). Meanwhile real machines accumulate stale CE copies through manual installs and plugin marketplaces, and harnesses that index both a user skills directory and the managed directory pay a per-session token tax for duplicated descriptions. Full problem frame, actors, flows, and acceptance examples live in the origin document.

---

## Requirements

Trace to origin (docs/brainstorms/2026-08-24-canonical-skills-adoption-requirements.md). Condensed; origin is authoritative.

- **Harvest & destination**: R1 harvest top-level `skills/`; R2 adaptive destination (adopt-in-place when `ce-*` surfaces exist, else canonical copy in the OpenCode managed directory only; adopted surfaces complete to the full canonical set); R3 explicit first-adoption confirmation with decline semantics; R4 no skill files into harness-owned dirs of harnesses without pre-existing `ce-*` skills.
- **Registry & resolution**: R5 registry indexes canonical/adopted surfaces wherever they live, refreshed per sync; R6 resolution serves canonical skills to any harness, falling back to canonical paths.
- **Verification & matrix**: R7 hash-verify every managed skills surface (`verified N/N`); R8 unmanaged harnesses keep `registered` wording with canonical-copy guidance; R12 `external-duplicate` state for marketplace-origin copies (never adoptable); R17 `pending-adoption` state for non-interactive contexts; R16 `restored-drift` reporting for user-edited managed files; R19 status/doctor surface adoption states; orphaned adopted root requires re-adoption.
- **Safety**: R9 rewrites restricted to `ce-*`-named dirs; R10 every rewrite backed up and restorable; R13 retirement of a harness's previous managed surface (manifest/ledger-tracked files only, with backup); R14 adoption confirms the path matches the harness's skills-root convention, else warn-and-skip; R15 transactional adoption (stage → apply → ledger atomically last → auto-restore on failure, non-zero exit).
- **Docs**: R11 user guide explains the canonical/adoption model, the adopt command, and on-demand resolution.

**Origin actors:** A1 multi-harness developer, A2 ce-ai, A3 harnesses.
**Origin flows:** F1 first adoption via the explicit adopt command; F2 on-demand resolution for copy-less harnesses.
**Origin acceptance examples:** AE1 (adopt stale opencode surface → verified 33/33, no managed-dir copy), AE2 (fresh claude → registered, zero harness-dir writes), AE3 (ce-debug rewritten, my-own-skill untouched), AE4 (codex resolves canonical paths), AE5 (harvest stages top-level `skills/`), AE6 (registry resolves pi from its adopted root).

---

## Scope Boundaries

- No symlink farms; no per-harness copy creation (rejected direction B).
- No per-harness subset selection (rejected direction C).
- No management of marketplace/plugin-channel installs — detected and reported (`external-duplicate`), never modified or adopted.
- No deletion of user files, except ce-ai-authored ledger-tracked files during managed-surface retirement (R13).
- Other release assets (`.claude/commands`, `.agents/plugins`, …) are not harvested.

### Deferred to Follow-Up Work

- TUI adoption action (menu entry spawning `skills adopt`): the TUI already executes CLI vectors; v1 ships CLI-only adoption. Follow-up PR.
- `upgrade`-time adoption prompts UX: upgrade inherits sync's report-only behavior in v1.

---

## Context & Research

### Relevant Code and Patterns

- `src/commands/install.rs:25` and `src/commands/sync.rs:26` — `MANAGED_PREFIXES` gate the harvested set; `read_local_tree` walks the whole extracted tree, so adding a top-level `skills` prefix requires no cache changes. `find_source_root` (src/source/archive.rs) resolves the root containing `.opencode/`; top-level `skills/` is a sibling and reachable.
- `src/commands/sync.rs:334-504` — verification matrix: `SurfaceCheck`/`CheckStatus`/`verify_tree_against`; `skills_expected` filter (sync.rs:343-347) already keys off the `skills/` manifest prefix; adopted-surface checks slot into the per-harness loop (sync.rs:358-432). Matrix render helpers (`matrix_line`, `reconciliation_line`, `guidance_note_lines`) added by PR #230.
- `src/commands/uninstall.rs:229-257` — confirmed `remove_dir_all` on the managed dir and on harness skills dirs (claude/codex/copilot/grok/kimi/agy/pi/fx); custom mode already does surgical manifest-scoped removal with `prune_empty_dirs` (L117-141) — the pattern to extend.
- `src/state/backups.rs` `backup_file` (file-level, timestamped) and `src/state/journal.rs` (`Journal::arm` records prior bytes before mutation; fault injection via `CE_AI_FAIL_AFTER_WRITES`) — the transactional rewrite toolkit; install.rs and sync.rs already wrap writes in `arm!`.
- `src/source/registry.rs` — `scan_skill_directory(dir, scope, target_harness, roots, map)`; `build()` Tier-3 scans `~/.ce-ai/harness-<kind>/skills` (dead root, no writer); `process_skill_file` with `target_harness: None` maps one path to all harnesses (the canonical-store precedent at `~/.ce-ai/skills`); `collect_authorized_roots` already authorizes real harness dirs, so adopted roots pass R3 validation; `resolve()` degrades gracefully on hash mismatch.
- `src/harness/registration.rs:44-76` + `sync_skills_root` (sync.rs) — per-harness skills-root conventions (agy nests `config/skills`; pi has no MCP; cursor consumes none) — the path-validity table for R14.
- `src/commands/skills.rs` — existing `skills list/resolve` command family to extend with `adopt`.
- TUI contract test `every_tui_spawned_vector_satisfies_its_cli_contract` (src/tui.rs) — new CLI subcommands must not break spawned-vector parsing; TUI gains no adopt action in v1.
- `tests/cli.rs` hermetic fixtures (`ceai`, `ce_source`) — `ce_source` builds `.opencode/skills/ce-brainstorm`; needs a top-level `skills/` fixture variant.

### Institutional Learnings

- `docs/solutions/logic-errors/init-prj-created-file-clobber-on-re-adoption-2026-08-22.md` — when replacing persisted entries wholesale, preserve historical fields; test the re-run/idempotent path (adoption re-run must not clobber provenance).
- `docs/solutions/architecture/multi-harness-skill-registry-engine.md` — registry persistence via `write_atomic` + POSIX 0644; lifecycle gated on `!ctx.dry_run`; Tier-3 scanning must thread `Option<HarnessKind>` correctly or skills bleed across harnesses.
- `docs/solutions/bugfixes/native-per-harness-directory-resolution.md` — resolve targets via `HarnessKind::harness_dir`, never hardcode paths; leakage tests assert opencode pristine.
- `docs/solutions/multi-harness-support-implementation.md` — "Custom roots are user-owned: uninstall removes exactly the manifest-recorded files" — the invariant being generalized.
- `docs/solutions/workflow-issues/multi-agent-shared-git-config-contention-2026-08-22.md` — sibling worktrees share `.git/config`; self-contain git fixtures in tests.

---

## Key Technical Decisions

- **Adopted-surface ledger lives in `~/.ce-ai/state.json`, not InstallManifest**: the install manifest is opencode-managed-dir-specific; adoption spans all harnesses and must survive `uninstall --harness opencode`. New state section `skill_surfaces: [{harness, root, files:[{path,sha256}], status: adopted|declined|orphaned, adopted_at}]`, persisted via `State::save` (atomic). Install manifest keeps tracking only the opencode managed surface.
- **Harvest via prefix extension**: add `skills` to `MANAGED_PREFIXES` (both files); managed-relative paths keep the `skills/` prefix so the sync `skills_expected` filter and matrix verification work unchanged for the managed surface.
- **Adoption is command-driven, never prompt-driven**: new `ce-ai skills adopt [--harness <name|all>] [--yes]` lists adoptable surfaces (detection: directory named `ce-*` containing `SKILL.md`, under the harness's canonical skills root per the registration table; R14 path-validity check), confirms per surface unless `--yes`, then runs the transactional rewrite. Sync/install never prompt; they report `pending-adoption`.
- **Transactional adoption (R15)**: stage canonical content → per-file `backup_file` → `Journal::arm` → `write_atomic` each file (completing partial sets) → write the ledger entry atomically last → on any failure, restore all just-written backups, leave the surface unadopted, exit `CeError::Runtime`.
- **Retirement (R13)**: adopting a surface for a harness that already has a managed surface — ledger-tracked adopted roots **or** the manifest-tracked managed-dir `skills/` tree — removes only ce-ai-authored tracked files (each backed up first; InstallManifest rewritten when applicable), then prunes empty `ce-*` dirs.
- **Uninstall rework**: for harness skills directories, replace `remove_dir_all` with ledger/manifest-scoped removal + `prune_empty_dirs` (custom-mode precedent), preserving non-CE content. The managed dir keeps whole-dir removal (ce-ai-owned).
- **Registry strategy — ledger-driven precision**: scan the managed-dir skills tree plus ledger-tracked adopted surfaces with `target_harness: None` (all-harness mapping, `~/.ce-ai/skills` precedent); drop the dead `~/.ce-ai/harness-<kind>/skills` Tier-3 scan. Copy-less harnesses resolve canonical paths through the normal `resolve()` flow (R6).
- **Matrix states**: extend the surface model with `pending-adoption`, `external-duplicate` (ce-* content found under known plugin-cache roots, e.g. Claude `plugins/cache`), and `restored-drift` reporting; `verified N/N` for ledger-tracked adopted surfaces; `registered` unchanged for skill-less harnesses.
- **External-origin heuristic (R18)**: `ce-*` directories under harness skills roots that are not ledger-tracked → `pending-adoption` (adoptable); CE content under plugin-cache/marketplace roots → `external-duplicate` (reported with paths, never adoptable). Refinement of origin detection deferred to implementation if the cache-root scan proves brittle.

---

## Open Questions

### Resolved During Planning

- Detection heuristic: `ce-*` directory prefix + `SKILL.md` present + **frontmatter name matching the canonical harvested set** (mandatory — protects user-authored `ce-*` skills); symlinked entries rejected; stale/current classification via hash compare against the harvest.
- Adopted-surface tracking: state.json ledger (rationale above), not InstallManifest.
- R6 mechanics: ledger-driven registry indexing + existing `resolve()`; `merge_skills_path` config registration already covers opencode; no new config mutation needed for v1.
- Flow-gap amendments (transactionality, drift reporting, non-interactive semantics, external-duplicate state, status/doctor visibility, retirement scope carve-out): folded into the origin document as R15-R19 before planning.

### Deferred to Implementation

- Final `CheckStatus` variant shape for the new matrix states (extend enum vs. `NotVerified` reason strings) — decided against the PR #230 render-helper API.
- External-origin detection tuning if plugin-cache layouts vary across harness versions.
- Exact ledger schema field names after touching `state.rs`.

---

## Implementation Units

### U1. Harvest the top-level skills tree into the canonical managed surface

**Goal:** `install`/`sync` harvest the release's top-level `skills/` tree; fresh machines get the canonical copy in the OpenCode managed directory.

**Requirements:** R1, R2 (fallback arm), R7 (managed surface), AE5.

**Dependencies:** None.

**Files:**
- Modify: `src/commands/install.rs`, `src/commands/sync.rs` (MANAGED_PREFIXES; remove `copy_managed_skills` call sites), `src/harness/registration.rs` (drop the now-unused helper if orphaned)
- Modify: `Dockerfile.e2e`, `e2e_runner.sh` (top-level `skills/` tree in the container fixture)
- Test: `tests/cli.rs` (fixture + harvest cases)

**Approach:**
- Extend `MANAGED_PREFIXES` with `skills`; managed-relative paths keep the `skills/` prefix. Precedence when a release ships both layouts: top-level `skills/` wins over legacy `.opencode/skills`; overlapping managed-relative paths warn instead of silently keeping the sort-order survivor.
- **Remove the `copy_managed_skills` call sites** at `install.rs:294` and `sync.rs:272` (registration-spec arm): harvested skills never flow into harness-owned directories — adoption (U2/U3) is the only delivery path into harness roots.
- Extend the `ce_source` fixture (or add `ce_source_with_top_level_skills`) so tests exercise the real release layout.
- Extend the container fixture: `Dockerfile.e2e` + `e2e_runner.sh` gain the top-level `skills/` tree so `make e2e` exercises harvest.

**Execution note:** Test-first — fixture + failing install test before the prefix change.

**Patterns to follow:** existing `install_fresh_install_creates_backup_entry_loader_skills_and_manifest`.

**Test scenarios:**
- Covers AE5. Happy path: release tree with top-level `skills/ce-brainstorm/SKILL.md` → install writes `compound-engineering/skills/ce-brainstorm/SKILL.md`, manifest gains the `skills/...` entry.
- Covers AE2. Happy path: fresh-machine sync → zero skill writes outside the OpenCode managed directory; claude/codex/… harness dirs untouched.
- Happy path: sync on a fresh machine → matrix shows `opencode: verified N/N` including skills files.
- Edge case: release without top-level `skills/` → harvest no-ops, install still succeeds (loader-only), no error.
- Edge case: release with both `.opencode/skills` and top-level `skills/` → top-level wins, overlap warning emitted.
- Integration: `make e2e` container gate exercises the harvest path end-to-end.

**Verification:** install + sync green with the new fixture; manifest lists skills entries; no harness-owned directory receives skill files.

### U2. Adoption detection, ledger schema, and the `skills adopt` command

**Goal:** Detect adoptable `ce-*` surfaces per harness, persist adoption state, and ship the explicit `skills adopt` command with confirmation, `--yes`, and decline/pending semantics.

**Requirements:** R2, R3, R4, R9, R14, R17; AE1 (detection/report parts).

**Dependencies:** U1.

**Files:**
- Modify: `src/state/state.rs` (ledger schema), `src/commands/skills.rs`, `src/commands/mod.rs`, `src/main.rs` (subcommand wiring)
- Create: `src/commands/adopt.rs` (detection + command logic)
- Test: `tests/cli.rs`, inline unit tests for detection/path-validity

**Approach:**
- `skill_surfaces` ledger in state.json (atomic via `State::save`); statuses `adopted | declined | orphaned`.
- Detection walks each harness's canonical skills root (registration table + `sync_skills_root`), flags `ce-*` dirs containing `SKILL.md`, and validates the root convention (R14) — warn-and-skip on mismatch. **Adoptability requires the SKILL.md frontmatter name to match a skill in the canonical harvested set** — `ce-*` dirs whose frontmatter name is not canonical are reported as unrecognized and skipped with a warning (protects user-authored `ce-*` skills per R9). Symlinked `ce-*` entries (dir or files) are rejected as adoptable and reported, never rewritten.
- `skills adopt` lists candidates (paths, stale vs current vs canonical hashes), confirms per surface unless `--yes`; non-TTY without `--yes` fails with `CeError::Usage` guidance. Decline records status `declined` (no re-prompt).
- No writes to harness dirs in this unit — rewrite engine lands in U3.

**Patterns to follow:** `src/commands/skills.rs` command family; `uninstall.rs --yes` flag handling; hermetic `ceai()` fixtures.

**Test scenarios:**
- Covers AE1 (detection). Happy path: opencode user dir with stale `ce-*` dirs → `skills adopt` lists the surface with stale/current classification.
- Edge case: `ce-*` dir outside the harness's canonical root (R14 mismatch) → warned and skipped, never adopted.
- Error path: non-TTY without `--yes` → `Usage` error with guidance; `--yes` proceeds.
- Happy path: decline → ledger records `declined`; repeated `skills adopt` offers it again (explicit re-run), sync never re-prompts.

**Verification:** `skills adopt` lists/records correctly; ledger persists across invocations; no skill files written yet.

### U3. Transactional rewrite engine: stage, backup, complete, retire, auto-restore

**Goal:** Execute an adoption transactionally — complete partial sets, back up and rewrite `ce-*` files, retire a superseded managed surface, write the ledger atomically last, and auto-restore on any failure.

**Requirements:** R2 (completion), R10, R13, R15; AE1 (apply parts), AE3.

**Dependencies:** U2.

**Files:**
- Modify: `src/commands/adopt.rs`, `src/state/journal.rs` (reuse), `src/state/backups.rs` (reuse)
- Test: `tests/cli.rs` (incl. journal fault injection), inline unit tests

**Approach:**
- Stage canonical files → per-file `backup_file` → `Journal::arm` → `write_atomic` → ledger write last.
- Completion: missing canonical `ce-*` skills are written alongside rewritten ones (set reaches the canonical count).
- Retirement: **any** prior managed surface for the same harness — ledger-tracked adopted roots **or** the manifest-tracked managed-dir `skills/` tree — is removed file-by-file (each backed up; InstallManifest rewritten when the managed surface is retired) before recording the new surface, so at most one indexed surface per harness remains (R13; prevents the double-description tax for opencode, the most common adoption target).
- The rewrite engine errors rather than rewriting any non-regular file (symlinks are rejected at detection; this is the backstop).
- Failure at any point → restore from this run's backups, surface stays unadopted, `CeError::Runtime`.

**Execution note:** Test-first on the failure path using `CE_AI_FAIL_AFTER_WRITES` fault injection.

**Patterns to follow:** journal `arm!` usage in install.rs/sync.rs; custom-mode surgical removal in uninstall.rs.

**Test scenarios:**
- Covers AE1, AE3. Happy path: adoption rewrites stale `ce-debug`, leaves `my-own-skill` untouched, completes missing canonical skills, ledger records full set with hashes.
- Covers R13. Happy path: harness with an existing managed surface adopts a new location → old tracked files removed (backed up), one surface remains.
- Covers R13. Happy path: adopting an opencode user-dir surface over an existing manifest-tracked managed-dir skills tree → managed-dir skills files retired (backed up, manifest updated), exactly one resolve() path per skill remains.
- Error path: fault injection mid-rewrite → all files restored to prior bytes, ledger has no adopted entry, exit non-zero.
- Edge case: re-adoption of an already-adopted surface → idempotent, provenance (`adopted_at`) preserved per the clobber learning.

**Verification:** adoption is atomic under fault injection; ledger and disk agree after every scenario.

### U4. Sync routine rewrites and drift restore on adopted surfaces

**Goal:** `sync` keeps adopted surfaces current: rewrites on upstream change, restores user edits as reported drift.

**Requirements:** R3 (automatic rewrites), R16, R7.

**Dependencies:** U3.

**Files:**
- Modify: `src/commands/sync.rs`
- Test: `tests/cli.rs`, `src/commands/sync.rs` unit tests

**Approach:**
- Sync reads ledger-tracked surfaces, diffs against canonical harvest, rewrites drifted/missing files (backup + `arm!`), reports each user-edit restore as `restored-drift` in the matrix output.

**Test scenarios:**
- Happy path: user edits an adopted `SKILL.md` → sync restores canonical content, output reports `restored-drift` with the path.
- Happy path: canonical content changes (new fixture hash) → sync rewrites the adopted surface.
- Edge case: adopted root deleted by the user → surface reported `orphaned` (R19), sync does not recreate silently.

**Verification:** adopted surfaces converge to canonical on every sync; drift is visible, never silent.

### U5. Matrix verification of adopted surfaces and new states

**Goal:** The sync verification matrix hash-verifies ledger-tracked adopted surfaces (`verified N/N`) and renders `pending-adoption`, `external-duplicate`, `restored-drift`, and `orphaned` states.

**Requirements:** R7, R8, R12, R16 (report), R17, R18, R19 (matrix part); AE1 (matrix assertion).

**Dependencies:** U2 (ledger), U4 (drift restore).

**Files:**
- Modify: `src/commands/sync.rs` (surface model + render helpers)
- Test: `src/commands/sync.rs` unit tests, `tests/cli.rs`

**Approach:**
- Extend the surface model per the PR #230 render-helper API (`matrix_line` et al.); reconciliation counts include the new states.
- External-duplicate scan: known plugin-cache roots (Claude `plugins/cache`) probed for CE content; reported with paths, excluded from resolution.

**Test scenarios:**
- Covers AE1. Happy path: adopted opencode surface → `✓ opencode: verified 33/33` (fixture-scaled).
- Happy path: non-interactive sync with an adoptable surface → `pending-adoption` line + guidance note.
- Happy path: CE content under a plugin-cache root → `external-duplicate` with paths; `skills resolve` excludes it.
- Edge case: declined surface → `registered` with canonical-copy guidance (no re-prompt).

**Verification:** every origin matrix assertion (AE1/AE2) renders exactly; wording pinned by unit tests.

### U6. Uninstall scoping: ledger-tracked removal, never whole-dir nukes

**Goal:** Uninstall removes only ce-ai-tracked skill files on harnesses with adopted/managed surfaces, preserving user-authored content.

**Requirements:** R9, R13 (retirement parity), Scope Boundaries (no user-file deletion); AE3 (uninstall variant).

**Dependencies:** U2, U3.

**Files:**
- Modify: `src/commands/uninstall.rs`
- Test: `tests/cli.rs` (update existing per-harness uninstall cases + new preservation cases)

**Approach:**
- Harness skills dirs: replace `remove_dir_all` with ledger/manifest-scoped file removal + `prune_empty_dirs` (custom-mode precedent). Managed dir keeps whole-dir removal.
- **Ledger lifecycle:** uninstall deletes the `skill_surfaces` entries for every surface whose tracked files it removed — scoped to the targeted harness under `--harness <name>`, all entries under `--harness all` — so a clean uninstall never leaves phantom `orphaned` reports.
- Existing per-harness uninstall tests updated to seed ledger state.

**Test scenarios:**
- Covers AE3. Happy path: uninstall on harness with adopted surface + user's `my-own-skill` → tracked files removed, `my-own-skill` intact, empty `ce-*` dirs pruned.
- Happy path: uninstall with no adoption (legacy state) → behavior equivalent to today for managed content, nothing user-owned removed.
- Edge case: ledger references vanished files → removal skips missing paths, no error.
- Happy path: post-uninstall `doctor` reports no `orphaned` surfaces (ledger entries cleaned with the removal).

**Verification:** no test can make uninstall delete a non-tracked user file; existing leakage tests stay green.

### U7. Registry alignment and adoption visibility (status/doctor)

**Goal:** Registry indexes ledger-tracked surfaces + managed dir for all harnesses; resolution serves copy-less harnesses; status/doctor surface adoption states and orphaned surfaces.

**Requirements:** R5, R6, R18 (exclusion), R19; AE4, AE6.

**Dependencies:** U2 (ledger), U5 (states).

**Files:**
- Modify: `src/source/registry.rs`, `src/commands/status.rs`, `src/commands/doctor.rs`
- Test: `src/source/registry.rs` unit tests, `tests/cli.rs`

**Approach:**
- `build()` scans managed-dir skills + ledger-tracked adopted roots with `target_harness: None`; dead `~/.ce-ai/harness-<kind>/skills` Tier-3 scan removed; adopted roots added to `collect_authorized_roots`.
- `resolve()` needs no fallback branch — canonical entries already map to all harnesses.
- status/doctor read the ledger + matrix classification to report adopted / pending-adoption / declined / external-duplicate / orphaned.

**Test scenarios:**
- Covers AE6. Happy path: pi surface adopted → registry resolves pi skills from the adopted root (not `~/.ce-ai/harness-pi/skills`).
- Covers AE4. Happy path: codex with no local skills → resolve returns canonical adopted/managed paths with valid hashes.
- Edge case: adopted root vanished → doctor reports orphaned, requires re-adoption.
- Integration: `skills resolve` excludes `external-duplicate` paths.

**Verification:** byte-stable resolve tests extended to adopted surfaces; doctor/status show adoption state.

### U8. Documentation, changelog, and version

**Goal:** User guide explains the canonical/adoption model; release metadata updated.

**Requirements:** R11.

**Dependencies:** U1-U7 (documents final behavior).

**Files:**
- Modify: `docs/user-guide/sync-and-upgrade-mechanisms.md`, `CHANGELOG.md`, `Cargo.toml`, `README.md` (only if the docs map references the guide)

**Approach:**
- New guide section: canonical surface model, `skills adopt` flow, matrix states table, resolution for copy-less harnesses; follows Diátaxis and docs-styling guide.

**Test scenarios:**
- Test expectation: none — documentation-only unit; gates are `cargo fmt/clippy/test` unaffected and doc style compliance.

**Verification:** guide renders the new states and command; CHANGELOG entry under the new version; SemVer minor bump (new command + behavior). Additionally: the origin success criterion requires measuring the duplicated skill-description token overhead on the reference machine — capture the baseline before Phase 1 lands and re-measure after Phase 4, reporting the delta in the CHANGELOG/PR description.

---

## System-Wide Impact

- **Interaction graph:** `install`, `sync`, `upgrade` (delegates to sync), `uninstall`, `skills`, `status`, `doctor`, TUI-spawned CLI vectors, and the e2e Docker gate all touch the skills surface.
- **Error propagation:** adoption failures exit `Runtime` (1) after auto-restore; adopted-surface drift keeps the `Verification` (6) contract for genuinely failed surfaces; `pending-adoption`/`external-duplicate` are non-failing states.
- **State lifecycle risks:** ledger vs disk divergence (orphaned surfaces) is reported, never auto-repaired; journal covers crash-consistency of rewrites; state.json growth is bounded (33 file hashes per adopted surface).
- **API surface parity:** new `skills adopt` subcommand must parse under the global flag set; TUI spawned-vector contract test must stay green (TUI gains no adopt action in v1).
- **Integration coverage:** install → adopt → sync → uninstall full lifecycle in `tests/cli.rs`; `make e2e` fixture extended with a top-level `skills/` tree to exercise harvest in the container gate.
- **Unchanged invariants:** opencode config mutations (`plugin[]`, `skills.paths`) are untouched; custom-mode install/uninstall contract unchanged; exit-code mapping unchanged; user-config preservation (hard-gate #4) strengthened, not relaxed.

---

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| Upstream drops top-level `skills/` from a future release | Harvest no-ops with a warning; managed/adopted surfaces keep last-known content; matrix reports staleness |
| Uninstall rework regresses on legacy (non-adoption) states | Existing per-harness uninstall tests updated + new preservation tests; managed-dir removal unchanged |
| External-origin (marketplace) detection brittleness across harness versions | v1 scopes the scan to known plugin-cache roots; heuristic refinement deferred to implementation |
| PR size boundary (400 lines) vs feature breadth | Four chained PRs of ~400 lines each (Phased Delivery); each unit ~200 LOC per CONTRIBUTING §4 |
| Concurrent harness sessions reading during rewrites | `write_atomic` per file + journal; torn-read window accepted (single-writer CLI, short window) |
| PR #230 not merged before work starts | Explicit dependency — merge first; R8 wording depends on it |

---

## Phased Delivery

Sequencing rule: no released binary may offer adoption execution while harness-dir `remove_dir_all` remains — the transactional engine and the uninstall rework ship in the same PR.

### Phase 1 (PR 1)
- U1 + U2: harvest + adoption detection/ledger/`skills adopt` command. The command's execution path returns an explicit "adoption engine ships in the next release" error until Phase 2 (listing/declining work).

### Phase 2 (PR 2)
- U3 + U6: transactional rewrite engine + uninstall scoping — adoption execution and the destructive-window fix land together.

### Phase 3 (PR 3)
- U4 + U5: sync rewrites + matrix states.

### Phase 4 (PR 4)
- U7 + U8: registry alignment, visibility, docs, release metadata, token-overhead re-measurement.

---

## Documentation / Operational Notes

- User guide gains the canonical/adoption section (R11) following `docs/references/docs-styling.md`; README stays ≤ 100 lines (link, don't inline).
- CHANGELOG entry under the next minor version; SemVer minor bump (new subcommand + behavior change).
- e2e fixture update is operational: `Dockerfile.e2e` + `e2e_runner.sh` gain the top-level `skills/` tree so the container gate exercises harvest.

---

## Sources & References

- **Origin document:** [docs/brainstorms/2026-08-24-canonical-skills-adoption-requirements.md](../brainstorms/2026-08-24-canonical-skills-adoption-requirements.md)
- Related code: `src/commands/install.rs`, `src/commands/sync.rs`, `src/commands/uninstall.rs`, `src/source/registry.rs`, `src/state/state.rs`, `src/harness/registration.rs`
- Related PRs/issues: #230 (matrix wording — merge dependency)
- Learnings: `docs/solutions/logic-errors/init-prj-created-file-clobber-on-re-adoption-2026-08-22.md`, `docs/solutions/architecture/multi-harness-skill-registry-engine.md`, `docs/solutions/multi-harness-support-implementation.md`
