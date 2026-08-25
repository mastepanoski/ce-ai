# Spec: canonical-skills-adoption

## ADDED Requirements

### Requirement CSA-1: Harvest and adaptive destination (R1, R2, R4)

`install`/`sync` SHALL harvest the release's top-level `skills/` tree. When no harness skills directory on the machine contains adoptable `ce-*` folders, the canonical copy SHALL be written exclusively to the OpenCode managed directory; when adoptable surfaces exist, they SHALL be adopted in place and no managed-directory copy SHALL be created for that harness. No skill files SHALL be written into harness-owned directories of harnesses without pre-existing `ce-*` skills. Adopted surfaces SHALL be completed to the full canonical set.

#### Scenario: Fresh machine harvest

- **WHEN** install runs against a release tree containing top-level `skills/`
- **THEN** the managed directory contains `skills/...` entries recorded in the install manifest
- **AND** no harness-owned skills directory receives any file

#### Scenario: Both-prefix precedence

- **WHEN** a release tree contains both `.opencode/skills` and top-level `skills/`
- **THEN** top-level `skills/` wins and an overlap warning is emitted

### Requirement CSA-2: Adoption command and ledger (R3, R9, R14, R15, R17)

Adoption SHALL happen only through the explicit `skills adopt` command. A `ce-*` directory is adoptable only if its SKILL.md frontmatter name matches the canonical harvested set, it sits under the harness's skills-root convention, and it contains no symlinks; otherwise it is warned and skipped. First adoption SHALL require per-surface confirmation (`--yes` bypass; non-TTY without `--yes` exits Usage). Adoption SHALL be transactional: stage → per-file backup → journal → atomic writes (completing partial sets) → retire any prior managed surface (ledger-tracked or the manifest-tracked managed-dir `skills/` tree) → ledger written atomically last; any failure SHALL auto-restore this run's backups, leave the surface unadopted, and exit `Runtime`. Declined surfaces stay untouched, render `registered`, and are not re-prompted.

#### Scenario: Adopt stale surface (AE1)

- **WHEN** `skills adopt` confirms a stale opencode user-dir surface
- **THEN** `ce-debug` is rewritten, `my-own-skill` untouched, missing canonical skills completed, the managed-dir skills tree retired, and the matrix shows `verified N/N` for the surface

#### Scenario: Fault mid-adoption

- **WHEN** a fault is injected after the first write
- **THEN** all files are restored from this run's backups, the ledger has no adopted entry, and the exit code is 1

### Requirement CSA-3: Sync currency and drift reporting (R3, R16)

`sync` SHALL rewrite ledger-tracked adopted surfaces to canonical (backup + journal per file) and SHALL report each user-edit restore as `restored-drift` with its path.

#### Scenario: User edit restored

- **WHEN** an adopted `SKILL.md` is manually edited and sync runs
- **THEN** canonical content is restored, a backup of the edit exists, and the output reports `restored-drift`

### Requirement CSA-4: Matrix states (R7, R8, R12, R17, R18, R19)

The matrix SHALL hash-verify every ledger-tracked adopted surface (`verified N/N`), render `pending-adoption` for adoptable-but-unadopted surfaces in non-interactive runs, `external-duplicate` (with paths, excluded from resolution) for CE content under known plugin-cache roots, and `orphaned` for vanished adopted roots. Skill-less harnesses keep `registered` with the canonical-copy guidance note. `status` and `doctor` SHALL surface adoption states.

#### Scenario: Pending adoption

- **WHEN** sync runs non-interactively with an adoptable surface
- **THEN** the surface renders `pending-adoption` with adoption guidance and nothing is written to it

#### Scenario: External duplicate

- **WHEN** CE content exists under a plugin-cache root
- **THEN** it renders `external-duplicate` with paths and `skills resolve` excludes it

### Requirement CSA-5: Scoped uninstall (R9, R13)

`uninstall` SHALL remove only ledger/manifest-tracked skill files on harness skills directories (pruning empty `ce-*` dirs), SHALL delete the ledger entries of removed surfaces (scoped to `--harness <name>`; all under `--all`), and SHALL NOT remove non-tracked user content. The managed directory retains whole-dir removal.

#### Scenario: Uninstall preserves user skills (AE3)

- **WHEN** uninstall runs on a harness with an adopted surface and a user-authored `my-own-skill`
- **THEN** tracked files and empty `ce-*` dirs are removed, `my-own-skill` survives, and `doctor` reports no `orphaned` surfaces

### Requirement CSA-6: Registry and resolution (R5, R6)

The SkillRegistry SHALL index the managed-dir skills tree plus ledger-tracked adopted surfaces with all-harness path mapping, refreshed after each sync; resolution SHALL serve canonical paths to any harness. The dead `~/.ce-ai/harness-<kind>/skills` scan SHALL be removed.

#### Scenario: Resolution from adopted root (AE6)

- **WHEN** pi's surface is adopted and the user resolves a skill for pi
- **THEN** the returned path points at the adopted root, not `~/.ce-ai/harness-pi/skills`

#### Scenario: Copy-less harness (AE4)

- **WHEN** codex has no local CE skills and the user resolves a skill
- **THEN** canonical, hash-valid paths are injected

### Requirement CSA-7: Documentation (R11)

The user guide SHALL explain the canonical/adoption model, the `skills adopt` flow, and the matrix states, self-served without external help.

#### Scenario: Guide coverage

- **WHEN** a reader opens the sync/upgrade guide
- **THEN** the canonical/adoption model, adopt command, and all matrix states are documented
