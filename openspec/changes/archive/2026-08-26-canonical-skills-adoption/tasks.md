# Tasks: canonical-skills-adoption

Work units carry per-unit changed-line estimates (~200 LOC target per CONTRIBUTING §4; rescopes may only narrow). Delivery: four chained PRs — no released binary may offer adoption execution while harness-dir `remove_dir_all` remains (U3+U6 ship together).

## PR 1 — Harvest + adoption surface detection

- [x] **U1 (~180 LOC). Harvest top-level `skills/` into the canonical managed surface.** Extend `MANAGED_PREFIXES` (install.rs, sync.rs) with top-level-wins precedence + overlap warning; remove `copy_managed_skills` call sites (install.rs:294, sync.rs:272) so harness-owned dirs receive nothing; `Dockerfile.e2e` + `e2e_runner.sh` fixture gains top-level `skills/`. Test scenarios: AE5 harvest; AE2 zero harness-dir writes; missing-tree no-op; both-prefix precedence; e2e integration. [Covers SCSA-1] — Shipped in #235 (26a129d).
- [x] **U2 (~200 LOC). Adoption detection, ledger schema, `skills adopt` command.** `skill_surfaces` ledger in state.rs (serde-default back-compat); detection = `ce-*` dir + SKILL.md + frontmatter ∈ canonical set + root-convention validity (R14) + symlink rejection; command lists candidates (stale/current vs canonical hashes), per-surface confirm / `--yes` / non-TTY Usage error; decline recorded (no re-prompt); execution path returns explicit "engine ships next release" until U3. Tests: detection incl. R14 mismatch + unrecognized frontmatter + symlink rejection; decline persistence; non-TTY error. [Covers CSA-2 partial] — Shipped in #235.
- [x] **Gates:** `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.

## PR 2 — Transactional engine + uninstall safety

- [x] **U3 (~200 LOC). Transactional rewrite engine.** Stage → backup_file each → Journal::arm → write_atomic (completes partial sets) → retire prior managed surface (ledger roots + manifest-tracked managed-dir `skills/` tree, each file backed up, InstallManifest rewritten) → ledger atomic last; failure auto-restores, `CeError::Runtime`. Tests: AE1 apply; R13 retirement incl. managed-dir case (one resolve path per skill); fault-injection rollback; idempotent re-adoption preserving provenance. [Covers CSA-2] — Shipped in #238 (76366e2).
- [x] **U6 (~160 LOC). Uninstall ledger-scoping.** Harness skills dirs: surgical tracked-file removal + `prune_empty_dirs` (custom-mode precedent); ledger entries deleted with their files (`--harness` scoped; all under `--all`); managed dir keeps whole-dir removal. Tests: AE3 preservation; legacy-state equivalence; vanished-path skip; post-uninstall doctor reports no orphaned. [Covers CSA-5] — Shipped in #238.
- [x] **Gates:** full gates as PR 1.

## PR 3 — Sync currency + matrix states

- [x] **U4 (~140 LOC). Sync rewrites + drift restore on adopted surfaces.** Ledger-driven diff vs canonical; rewrites (backup + journal); `restored-drift` reporting; vanished root ⇒ `orphaned`, never recreated. Tests: user-edit restore; upstream bump rewrite; orphaned reporting. [Covers CSA-3] — Shipped in #244 (0235800).
- [x] **U5 (~180 LOC). Matrix verification of adopted surfaces + new states.** `verified N/N` per adopted surface; `pending-adoption`, `external-duplicate` (plugin-cache probe, excluded from resolve), `restored-drift`, `orphaned` rendering via PR #230 helpers; reconciliation counts updated. Tests: AE1 matrix assertion; pending-adoption non-interactive; external-duplicate + resolve exclusion; declined ⇒ `registered`. [Covers CSA-4] — Shipped in #244.
- [x] **Gates:** full gates as PR 1.

## PR 4 — Registry, visibility, docs, release

- [x] **U7 (~180 LOC). Registry alignment + adoption visibility.** Scan managed-dir skills + ledger roots with `target_harness: None`; remove dead Tier-3; adopted roots into `collect_authorized_roots`; status/doctor report adoption states + orphaned. Tests: AE6 resolve from adopted root; AE4 copy-less harness; orphaned doctor report; external-duplicate resolve exclusion. [Covers CSA-6, CSA-4 visibility] — Shipped in #246 (f491ea6).
- [x] **U8 (~120 LOC). Docs + release.** User guide canonical/adoption section (R11) per docs-styling; CHANGELOG; Cargo.toml minor bump; token-overhead baseline (pre-Phase 1) and post-Phase-4 re-measurement reported in CHANGELOG/PR description. [Covers CSA-7] — Shipped in #246 + #248 (3c105d2, v1.25.0).
- [x] **Gates:** full gates as PR 1 + 100% green CI matrix per PR.
