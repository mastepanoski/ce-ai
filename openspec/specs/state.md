---
title: "State Management, Model Profiles & Configuration Overrides"
domain: state
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: State Management, Model Profiles & Configuration Overrides

## 1. Overview & Architectural Boundaries

The `state` module manages global persistence (`~/.config/ce-ai/state.json`), model slot assignments across agent personas (`ce-brainstorm`, `ce-plan`, `ce-work`), model snapshot profiles, and repository-local configuration overrides (`.ce-ai.json`).

## 2. Capabilities & Requirements

### R1. Atomic File Persistence (`write_atomic`)
WHEN saving state or writing configuration files  
THEN the system MUST write through a temporary sibling file and perform an atomic rename (`rename(2)` / POSIX atomic swap) to prevent corruption during unexpected termination.

### R2. Workspace Local Overrides (`.ce-ai.json`)
WHEN a repository defines `.ce-ai.json` at the root  
THEN `ce-ai` MUST deserialize local overrides (model assignments, `doc_hygiene`, `archive_compaction_threshold`) and merge them over global state with higher precedence.

### R3. Model Assignment Drift Reconciliation
WHEN model configurations change in host harness settings or `state.json`  
THEN `ce-ai sync` MUST bidirectionally reconcile assignments and update `state.json` cleanly.

### R4. Model Profile Snapshots
WHEN operator saves a profile snapshot (`ce-ai models profile save <name>`)  
THEN the system MUST snapshot all active model slots and allow restoring the snapshot via `ce-ai models profile load <name>`.

## 3. Data Models & CLI Contracts

- `State` struct: schema of `~/.config/ce-ai/state.json`.
- `ModelAssignment`: mapping agent slots (`brainstorm`, `plan`, `work`, etc.) to specific LLM models.
- `DocHygieneConfig`: `{ stale_spec_days, check_solution_paths, require_solution_frontmatter, archive_compaction_threshold }`.
- CLI commands: `ce-ai models set`, `ce-ai models list`, `ce-ai models profile save/load`.

## 4. Invariants & Operational Boundaries

- Direct unbuffered file overwrites of `state.json` or `opencode.json` are strictly forbidden (Hard Invariant #3).
- Model assignments MUST validate slot names against recognized canonical roles.
