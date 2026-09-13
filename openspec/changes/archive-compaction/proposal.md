# Proposal: Generational Archive Compaction & Milestone Rollups (`ce-ai archive compact`)

## Problem Statement
In Compound Engineering, `openspec/changes/archive/` serves as the project's permanent audit record linking shipped releases to their frozen contracts. Over prolonged development cycles, loose package directories accumulate rapidly:
1. **Archive Directory Sprawl:** `ce-ai` currently hosts 109 separate change package directories in `openspec/changes/archive/` comprising >540 markdown files.
2. **Context & Search Dilution:** AI coding agents and local tools conducting repository searches (e.g. `grep_search`, `find_by_name`, CodeGraph exploration) frequently ingest hundreds of historical scaffolding files (`exploration.md`, `tasks.md`), burning LLM tokens and cluttering search hits.
3. **Missing Lifecycle Management:** While completed specs must never be deleted (hard invariant: audit trail preservation), older historical packages can be rolled up into consolidated milestone summaries while packaging raw scaffolding into compressed tarballs.
4. **Diagnostic Invisibility:** `ce-ai doctor` does not currently monitor archive volume, leaving teams unaware of escalating archive sprawl until manual cleanup is triggered.

## In-Scope
1. **Generational Archive Compaction Command:**
   - Implement `ce-ai archive compact` and alias `ce-ai workflow archive compact`.
   - CLI flags: `--before <YYYY-MM-DD>`, `--milestone <name>`, `--threshold <N>`, `--dry-run`, `--no-tarball`, `--keep-loose`.
2. **Structured Milestone Rollup Generation:**
   - Groups candidates by calendar quarter (e.g. `2026-Q1`, `2026-Q2`, `2026-Q3`) or user-specified milestone tag.
   - Generates consolidated rollup documents at `openspec/changes/archive/milestones/<milestone>.md` containing feature catalogs, problem summaries, spec acceptance criteria, and task completion metrics.
3. **Safe Scaffolding Tarball Packaging:**
   - Bundles raw feature folders into `openspec/changes/archive/milestones/archive-<milestone>.tar.gz` with verified relative paths.
   - Safely removes loose compacted directories after verifying tarball integrity (unless `--keep-loose` is requested).
4. **Audit Ledger Synchronization:**
   - Updates `openspec/changes/archive/README.md` with a `## Compacted Milestones` ledger table referencing rollup markdown files and tarballs.
5. **Doctor Diagnostic Health Check:**
   - Adds `probe_archive_compaction` checking uncompacted count against `doc_hygiene.archive_compaction_threshold` (default: 30) in `.ce-ai.json`.
   - Surfaces non-blocking `doctor-warn:` advice in `ce-ai doctor` with copy-pasteable CLI commands.
   - Surfacing in `DocDebtReport` and Turn-0 `summary_line`.
6. **Comprehensive Test Suite:**
   - Unit tests for date parsing, candidate filtering, markdown rollup generation, tarball creation, ledger updating, and doctor diagnostics.
   - Integration tests executing real CLI compaction workflows in isolated temporary environments.

## Out-of-Scope
- Deleting historical specifications without generating milestone rollups and tarballs (forbidden by audit invariants).
- Modifying active pending changes in `openspec/changes/` (compaction is strictly restricted to `openspec/changes/archive/`).
- Automated background compaction without explicit user CLI command invocation (doctor strictly diagnoses and advises).
- External network dependencies or remote storage upload.

## Risk Evaluation & Mitigation
- **Risk (Data Loss during Pruning):** Loose directories deleted before rollup/tarball is safely committed to disk.
  - *Mitigation:* Generate rollup markdown and tarball first; verify tarball size and file entry count; only then remove loose directories.
- **Risk (Broken Historical Traceability):** External issues or PRs referencing historical change directories.
  - *Mitigation:* Milestone rollups retain identical slug headers and anchor links; `README.md` links directly to milestone summaries and tarball archives.
- **Risk (No-Git Environments):** Date resolution fails when `.git` is absent.
  - *Mitigation:* Multi-tier date extraction: (1) leading `YYYY-MM-DD-` folder prefix, (2) git commit date, (3) filesystem `mtime` fallback.

## Success Criteria
1. `ce-ai archive compact --dry-run` accurately plans compaction and displays preview without touching disk.
2. `ce-ai archive compact` consolidates target historical packages into `milestones/<milestone>.md` and `.tar.gz`, updating `README.md`.
3. `ce-ai doctor` alerts when uncompacted archive packages exceed 30 and recommends `ce-ai archive compact`.
4. 100% green verification: `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo test`.
