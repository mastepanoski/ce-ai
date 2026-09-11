# Proposal: Fix `init-prj` dry-run output (misleading adoption + omitted hook preview)

## Problem
`ce-ai --dry-run init-prj` is correctly non-destructive, but it reports the **same success line as a real run** (`✓ Adopted project ...`) and omits the RTK hook actions a real run performs. Users cannot trust dry-run to preview impact. Found while dogfooding `init-prj` on the `ce-ai` repo itself (issue #351).

## In Scope
- Honest, explicit dry-run reporting for `init-prj` (`dry-run: would adopt project ...`).
- Surface planned RTK hook configuration in dry-run by invoking the already dry-run-aware reconciliation path.
- Regression test locking in: dry-run writes zero files.

## Out of Scope
- Adding dry-run awareness to every non-dry-run-aware reconciliation helper (`reconcile_project_harness_hooks`, `ensure_gitignore_block`, `SkillRegistry::sync_registry`, `init_codegraph_if_available`) — tracked as a follow-up.
- `deinit-prj` dry-run behavior.

## Risks
- Moving/invoking RTK reconciliation in dry-run must not mutate anything. `configure_rtk_hook` returns before any write when `dry_run=true` (`src/harness/rtk.rs:117-122`), so the risk is contained.
- Real-run ordering must remain byte-for-byte behavior-equivalent; only a dry-run-only call is added.

## Success Criteria
- `--dry-run` output never claims adoption and clearly marks itself as a preview.
- Dry-run prints `[dry-run] would configure rtk hook for <harness>` for supported detected/installed harnesses.
- Dry-run leaves the target repo and `state.json` byte-identical and exits `0`.
- Real-run stdout and side effects are unchanged.
