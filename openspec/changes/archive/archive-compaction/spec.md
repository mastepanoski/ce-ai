# Specification: Generational Archive Compaction & Milestone Rollups

## Scenario 1: Archive Compaction Dry-Run Preview
- **WHEN** `ce-ai archive compact --dry-run` is invoked:
  - **THEN** it resolves all loose packages in `openspec/changes/archive/`.
  - **AND** it groups eligible packages by milestone (default quarterly or `--milestone`).
  - **AND** it displays the planned rollup summary paths and tarball paths to stdout.
  - **AND** it does NOT create `milestones/`, write any markdown or tarball files, or delete any loose directories.

## Scenario 2: Standard Compaction Execution & Verification
- **WHEN** `ce-ai archive compact` is invoked without `--dry-run`:
  - **THEN** it writes a consolidated milestone rollup markdown file to `openspec/changes/archive/milestones/<milestone>.md`.
  - **AND** it packages raw package directories into `openspec/changes/archive/milestones/archive-<milestone>.tar.gz`.
  - **AND** it verifies that the tarball is non-empty and readable.
  - **AND** it removes the loose package directories that were rolled up.
  - **AND** it updates `openspec/changes/archive/README.md` with a milestone ledger row linking to the rollup doc and tarball.

## Scenario 3: Date Filtering via `--before`
- **WHEN** `ce-ai archive compact --before <YYYY-MM-DD>` is invoked:
  - **THEN** only packages with resolved dates strictly prior to the cutoff date are selected for compaction.
  - **AND** packages dated on or after the cutoff remain untouched as loose directories in `openspec/changes/archive/`.

## Scenario 4: Explicit Milestone Tagging via `--milestone`
- **WHEN** `ce-ai archive compact --milestone <name>` is invoked:
  - **THEN** all selected candidates are assigned to the specified milestone name `<name>`.
  - **AND** the rollup document is written to `openspec/changes/archive/milestones/<name>.md`.
  - **AND** the tarball is named `openspec/changes/archive/milestones/archive-<name>.tar.gz`.

## Scenario 5: Flag Controls `--keep-loose` and `--no-tarball`
- **WHEN** `ce-ai archive compact --keep-loose` is invoked:
  - **THEN** rollup markdown and tarballs are generated normally.
  - **AND** the original loose directories in `openspec/changes/archive/` are preserved without deletion.
- **WHEN** `ce-ai archive compact --no-tarball` is invoked:
  - **THEN** rollup markdown is generated.
  - **AND** no `.tar.gz` tarball is created.

## Scenario 6: Doctor Diagnostic Health Check & Threshold Gate
- **WHEN** `ce-ai doctor` is executed and loose packages in `openspec/changes/archive/` exceed `archive_compaction_threshold`:
  - **THEN** `probe_archive_compaction` returns `ProbeStatus::Debt(ArchiveCompactionFinding)`.
  - **AND** `ce-ai doctor` emits a non-blocking advisory line:
    `doctor-warn: archive has <N> uncompacted packages (><threshold> threshold); run 'ce-ai archive compact' to roll up aged changes into milestone summaries`.
- **WHEN** loose packages are below or equal to `archive_compaction_threshold`:
  - **THEN** `probe_archive_compaction` returns `ProbeStatus::Clean`.

## Scenario 7: Multi-Tier Date Resolution & Fallbacks
- **WHEN** a folder name starts with `YYYY-MM-DD-`:
  - **THEN** the date is extracted directly from the name without querying git or filesystem mtime.
- **WHEN** a folder lacks a date prefix but git is available:
  - **THEN** the date is resolved from `git log -1 --format=%cs -- <path>`.
- **WHEN** a folder lacks a date prefix and git is unavailable:
  - **THEN** the date is resolved from filesystem `mtime`.
