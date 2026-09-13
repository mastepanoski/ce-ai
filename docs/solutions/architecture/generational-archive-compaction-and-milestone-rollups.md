---
title: "Generational Archive Compaction and Milestone Rollups"
category: "architecture"
date: "2026-09-12"
problem_type: "architecture"
tags:
  - archive
  - compaction
  - milestones
  - doc-debt
  - doctor
  - tarball
  - openspec
components:
  - commands::archive_compact
  - commands::workflow
  - commands::doctor
  - state::state
applies_when: "Compacting aged OpenSpec archive packages into milestone summaries or resolving doctor archive compaction debt"
---

# Generational Archive Compaction and Milestone Rollups

## Context

As a repository accumulates completed features, the `openspec/changes/archive/` directory steadily grows. In active projects with dozens or hundreds of completed initiatives, loose directories create significant friction:
1. **Agent Traversal Bloat**: AI coding agents inspecting `openspec/` or running directory searches suffer token waste and context dilution from hundreds of historical task files.
2. **Filesystem Clutter**: Many small directories increase inode usage and make manual repository auditing cumbersome.
3. **Loss of High-Level Narrative**: Individual change folders capture granular task checklists but lack quarterly or milestone-level executive summaries that explain how features rolled out over time.

To solve this sustainably without deleting historical verification artifacts, `ce-ai` implements **Generational Archive Compaction & Milestone Rollups** (`ce-ai archive compact` / `ce-ai workflow archive compact`).

## Architecture & Implementation

### 1. Dedicated Modular Engine (`src/commands/archive_compact.rs`)
Rather than expanding `src/commands/workflow.rs`, the compaction lifecycle is encapsulated within a dedicated module:
- `ArchiveSubcommand::Compact(CompactArgs)`: Integrated as a subcommand under `ce-ai archive compact` and `ce-ai workflow archive compact`.
- CLI flags support full operational control:
  - `--before <YYYY-MM-DD>`: Compaction cutoff filter.
  - `--milestone <name>`: Explicit rollup target name (bypassing default quarterly grouping).
  - `--threshold <N>`: Inline compaction trigger gate.
  - `--dry-run`: Read-only preview of candidates, rollups, and tarballs.
  - `--keep-loose`: Retain raw directories while generating rollups.
  - `--no-tarball`: Synthesize markdown summaries without generating `.tar.gz` archives.

### 2. Multi-Tier Resilient Date Resolution
To group candidates into quarters (e.g., `2026-Q3`), dates are resolved using a deterministic multi-tier strategy:
1. **Folder Prefix**: If the folder begins with `YYYY-MM-DD-`, the date is parsed directly without external process overhead.
2. **Git History**: If the folder name lacks a date prefix, `git log -1 --format=%cs -- <path>` is queried.
3. **Filesystem Fallback**: If git is unavailable or returns empty, the directory's filesystem `mtime` is used.

### 3. Safe Two-Phase Compaction Invariants
Data integrity is paramount when pruning historical artifacts:
1. **Consolidated Rollup Markdown**: Generates `openspec/changes/archive/milestones/<milestone>.md` summarizing feature proposals, primary specifications, task completion counts, and references to compressed bundles.
2. **Atomic Tarball Creation & Verification**: Raw directories are bundled into `openspec/changes/archive/milestones/archive-<milestone>.tar.gz` using standard `flate2` and `tar`.
3. **Integrity Verification Before Deletion**: Before removing any loose directory, `verify_milestone_tarball` parses the generated archive, confirming that every candidate folder is present and non-empty. Loose folders are pruned only after verification succeeds (and `--keep-loose` is not specified).
4. **Atomic Ledger Synchronization**: Updates `openspec/changes/archive/README.md` under `## Compacted Milestones` using `crate::state::write_atomic`, recording the milestone name, compacted count, date, and tarball link.

### 4. Continuous Health Guardrails (`probe_archive_compaction` & Doctor)
- `DocHygieneConfig` includes `archive_compaction_threshold` (default: 30) configurable via `.ce-ai.json`.
- `ce-ai doctor` emits a non-blocking diagnostic warning when uncompacted archive packages exceed the threshold:
  ```text
  doctor-warn: archive has 42 uncompacted packages (>30 threshold); run 'ce-ai archive compact' to roll up aged changes into milestone summaries
  ```
- `DocDebtReport::summary_line` surfaces compaction debt alongside stale specs, desyncs, and dead solution links during Turn-0 agent delivery.

## Key Learnings & Operational Constraints

1. **Verify Before Pruning**: Never execute `fs::remove_dir_all` based on exit codes alone. Always read back the archive index from disk to ensure entries exist and are non-empty. Furthermore, pruning MUST only execute when a verified tarball exists (`!args.no_tarball`) and `--keep-loose` is false, preventing data loss when generating markdown-only rollups.
2. **Gzip Stream Finalization**: Calling `tar.finish()?` only writes the tar stream; the underlying `GzEncoder` must be explicitly finalized via `tar.into_inner()?.finish()?` to flush the gzip trailer (CRC32 and uncompressed size) and detect I/O errors prior to verification.
3. **UTF-8 Character Boundary Safety**: Never slice paths or user-authored markdown content using raw byte offsets (e.g. `[..10]`). Always check `is_char_boundary` or use `char_indices` to prevent runtime panics on multibyte Unicode characters (accents, emojis, em-dashes).
4. **Maintain Strict README Limits**: Command additions must keep root `README.md` at or under the 100-line hard invariant (`wc -l README.md <= 100`).
5. **Preserve Subcommand Invariants**: Wiring `ArchiveSubcommand` into Clap required careful propagation so both `ce-ai archive compact` and legacy `ce-ai archive <feature>` continue to function seamlessly.
