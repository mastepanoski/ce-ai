# Design: canonical-skills-adoption

## Architecture

New/extended surfaces (all paths repo-relative):

- `src/commands/adopt.rs` (new): adoptable-surface detection + `skills adopt` command + transactional rewrite engine (U2/U3).
- `src/state/state.rs`: `skill_surfaces` ledger on `State` — `Vec<SkillSurface>` with `SkillSurface { harness, root: PathBuf, files: Vec<ManifestFile>, status: adopted|declined|orphaned, adopted_at }`; serde defaults keep old state.json loading (no `deny_unknown_fields` today — verified).
- `src/commands/install.rs` / `src/commands/sync.rs`: `MANAGED_PREFIXES` gains `skills`; `copy_managed_skills` call sites removed (registration-spec arm); sync gains adopted-surface rewrites (U4), matrix states (U5), and `pending-adoption`/`external-duplicate`/`restored-drift`/`orphaned` rendering via the PR #230 helpers (`matrix_line`, `reconciliation_line`, `guidance_note_lines`).
- `src/commands/uninstall.rs`: harness skills-dir removal becomes ledger-scoped (surgical, `prune_empty_dirs`); ledger entries deleted with their files; managed dir keeps whole-dir removal (U6).
- `src/source/registry.rs`: `build()` scans managed-dir skills + ledger-tracked adopted roots with `target_harness: None`; dead `~/.ce-ai/harness-<kind>/skills` Tier-3 scan removed; adopted roots added to `collect_authorized_roots` (U7).
- `src/commands/status.rs` / `src/commands/doctor.rs`: adoption-state reporting incl. orphaned surfaces (U7).

## Data flow (adoption)

```
skills adopt [--harness N|all] [--yes]
  → detect: harness skills root ∩ ce-*/SKILL.md ∩ frontmatter ∈ canonical set (else warn-skip)
  → path-validity vs registration table (R14); symlink rejection
  → stage canonical files → backup_file each → Journal::arm → write_atomic each
     (completes partial sets) → retire prior managed surface (ledger roots +
     manifest-tracked managed-dir skills tree; each file backed up) →
  → State::save ledger (atomic, last) → registry refresh
  failure ⇒ restore this run's backups, surface unadopted, CeError::Runtime
```

> Directional guidance for review, not implementation specification.

## Key contracts

- **Ledger is the single adoption truth**: matrix verification, sync rewrites, uninstall scoping, and registry indexing all read `skill_surfaces`. A vanished root ⇒ `orphaned` (reported; re-adoption required) — never silently recreated.
- **One managed surface per harness (R13)**: adoption retires prior ledger roots AND the manifest-tracked managed-dir `skills/` tree (backed up, InstallManifest rewritten) before recording the new surface.
- **Token-neutrality (R4)**: no skill files are written into harness-owned directories by install/sync — adoption is the only delivery path. Fresh machines get the managed-dir copy only (opencode-visible via existing `skills.paths`).
- **Non-interactive (R17)**: adoptable surfaces render `pending-adoption` in the matrix; `skills adopt` without `--yes` on a non-TTY exits `CeError::Usage` with guidance.
- **External duplicates (R18)**: CE content under known plugin-cache roots (Claude `plugins/cache`) renders `external-duplicate` with paths; excluded from `resolve()`.
- **Exit codes**: adoption failure = `Runtime` (1); managed-surface drift = `Verification` (6); new states are non-failing.

## Test strategy

- Hermetic CLI integration (`tests/cli.rs`): new top-level `skills/` fixture; full lifecycle install → adopt → sync → uninstall; zero-harness-write assertion (AE2); fault-injection rollback (R15).
- Unit: detection/path-validity, matrix rendering pins, registry scan/resolve, ledger serde back-compat.
- Container: `Dockerfile.e2e` + `e2e_runner.sh` gain the top-level `skills/` tree (`make e2e` exercises harvest).
