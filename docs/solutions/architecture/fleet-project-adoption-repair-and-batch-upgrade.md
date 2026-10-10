---
module: src/commands/doctor.rs
date: '2026-10-10'
problem_type: architecture
category: architecture
component: project_adoption
severity: medium
symptoms:
  - stale block versions reported across multiple registered projects in doctor --all-projects
  - manual loop required to re-run init-prj on each adopted project
root_cause: tooling_gap
resolution_type: code_fix
tags:
  - project-adoption
  - doctor-fix
  - fleet-governance
  - batch-upgrade
title: "Fleet Project Adoption Auto-Repair and Batch Upgrades"
applies_when: "When dealing with stale project adoption blocks across registered repositories or running doctor --all-projects."
---

# Fleet Project Adoption Auto-Repair and Batch Upgrades

## Problem

When `ce-ai` bumps the managed instruction block version (`BLOCK_VERSION`), registered project repositories tracked in `~/.ce-ai/state.json` retain their existing blocks until explicitly upgraded. Running `ce-ai doctor --all-projects` surfaces warnings such as `project-adoption: stale block version v=6 at '...' — re-run ce-ai init-prj --tier full to upgrade` or `project-adoption: block SHA drift detected`.

Previously, `ce-ai doctor --fix` repaired solution frontmatter, created missing spec folders, and archived completed changes, but could not automatically upgrade or repair registered projects. Users managing a fleet of adopted repositories had to manually invoke `ce-ai init-prj <path> --tier <tier>` for each repository one by one.

## Solution

In `ce-ai` v3.1.0, two complementary capabilities were added:

### 1. Auto-Repair in `ce-ai doctor --fix`

`src/commands/doctor.rs` now includes project adoption repair logic in `check_project_adoption_health`:
- When `args.fix` is set, any evaluated project with `StaleVersion`, `DriftDetected`, `FileMissing`, or missing harness hooks is automatically re-adopted via `init_prj::adopt_project`.
- When paired with `--all-projects`, `ce-ai doctor --fix --all-projects` scans the entire fleet registered in `state.projects` and brings all repositories up to the current `BLOCK_VERSION`, reconciling missing hooks without manual intervention.

### 2. Batch Upgrade via `ce-ai init-prj --all`

`src/commands/init_prj.rs` introduces the `--all` flag:
- Iterates over all registered projects in `state.projects`.
- Preserves each repository's configured tier (`full`, `minimal`, `orchestrator`) by default unless overridden via `--tier`.
- Re-applies managed blocks, derived stubs (`CLAUDE.md`), harness hooks, and RTK injection across the fleet in a single atomic invocation.

## Why This Works

Decoupling single-target adoption logic into a reusable `adopt_project` function allows both the diagnostic repair loop (`doctor --fix`) and the CLI command handler (`init-prj --all`) to share identical guarantees: atomic file writes, tier preservation, and cross-harness hook reconciliation.
