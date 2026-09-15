---
title: "OpenSpec Change Archival & Generational Compaction"
domain: archive
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: OpenSpec Change Archival & Generational Compaction

## 1. Overview & Architectural Boundaries

The `archive` subsystem provides safe, auditable transition of finished OpenSpec changes from `openspec/changes/<feature>` into `openspec/changes/archive/`, synchronizes the archive ledger, and executes generational compaction into milestone rollups.

## 2. Capabilities & Requirements

### R1. Dual Archival Criteria Validation
WHEN archiving a feature change via `ce-ai archive <feature>`  
THEN the system MUST validate completion using either:
- **Criterion 1 (Mechanical):** 100% of tasks in `tasks.md` are checked (`completed == total`).
- **Criterion 2 (STATUS-Attested):** When tasks were rescoped or cut, operator provides `--status "<explanation>"` or an explicit release commit reference.

### R2. Atomic Safe Mover & Collision Protection
WHEN moving an active change to `archive/`  
THEN the mover MUST verify destination non-existence, ensure clean working state, and update `archive/README.md` ledger atomically.

### R3. Generational Archive Compaction (`ce-ai archive compact`)
WHEN loose archived packages exceed the configured threshold (default: 30)  
THEN `ce-ai archive compact` MUST:
1. Aggregate packages into a milestone rollup markdown file (`openspec/changes/archive/milestones/<milestone>.md`).
2. Package raw exploratory folders into a compressed tarball (`archive-<milestone>.tar.gz`).
3. Verify tarball integrity before pruning loose directories.
4. Update `archive/README.md` with milestone links.

## 3. Data Models & CLI Contracts

- `ArchiveCriterion`: `Mechanical { completed, total }` or `StatusAttested { status, completed, total }`.
- `MilestoneRollup`: structured catalog of compacted features.
- CLI commands: `ce-ai archive <feature>`, `ce-ai archive --all`, `ce-ai archive compact`.

## 4. Invariants & Operational Boundaries

- Loose archive directories MUST NEVER be pruned if tarball creation fails or verification reports missing entries.
- Unarchived changes MUST be detected and surfaced by `doctor` and Turn-0 `workflow resume`.
