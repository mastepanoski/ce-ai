# Exploration

## Strategy decision

Staged generations require a single atomic switch point; install/sync targets
are heterogeneous files across per-vendor directories plus user-owned custom
roots, so there is no such point. Journal chosen (issue option 2).

## Ordering

`state.json` is already the final mutation in both commands today; the journal
preserves and *proves* that ordering (a fault injected at the state-save step
rolls the file back to its pre-command bytes).

## Fault injection surface

`CE_AI_FAIL_AFTER_WRITES=N` counted over `Journal::tracked_write` calls only —
deterministic because managed trees iterate a BTreeMap. Tests cover an early
cut (N=0: nothing applied, nothing to roll back) and a late cut (N large enough
to fail at/after config mutations but before state save), plus doctor flagging
and full recovery on the next run.
