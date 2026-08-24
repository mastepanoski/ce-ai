# Proposal: `openspec-hygiene-and-error-transparency`

## Why

Two debts from the v1.20.0 audit sweep:

1. **The OpenSpec ledger lies by omission.** There is no archive convention,
   so completed change folders accumulate forever in `openspec/changes/`;
   20 of 49 carry open task boxes, including three changes that this very
   series merged and shipped (#205/#206/#207). The audit series repeatedly
   flagged exactly this pattern ("tasks 0/13").
2. **Residual silent-failure sites** (`let _ =`) survive outside sync:
   18 in `deinit_prj.rs` alone, plus scattered best-effort ops in
   init_prj/install/uninstall/upgrade/sync. A failed de-adoption cleanup is
   invisible today; the user believes vendors were cleaned when they were
   not.

## What Changes

- Establish `openspec/changes/archive/` with a written convention and triage
  table; move every folder whose completion is mechanically verifiable
  (zero open boxes) or feature-level provable (STATUS header citing
  evidence); tick the three shipped-by-this-series folders first.
- Introduce two shared reporters (`report_best_effort_remove/_write`) and
  apply them across deinit_prj/init_prj/install/upgrade/sync/uninstall so
  unexpected cleanup failures warn on stderr while `NotFound` stays silent.
- Patch bump 1.20.1 + CHANGELOG.

## In Scope

- File moves + task ticks + new README under `openspec/`.
- The listed call-site conversions and the two helpers.
- One unit test per helper.

## Out of Scope

- Item-by-item audit of open boxes inside legacy archived folders
  (declared unaudited via STATUS headers instead).
- install.rs strategy-table consolidation (separate documented debt).
- Pruning stale local git branches (requires owner decision).
