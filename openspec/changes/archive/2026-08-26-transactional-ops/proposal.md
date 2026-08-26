# Proposal: `transactional-ops`

## Why

Issue #166 (P2): `write_atomic` protects individual files, but `install` and
`sync` mutate backups → managed files → harness configs → manifests →
registries → `state.json` **sequentially with no journal or rollback**. A
disk-full, permission error or crash mid-sequence leaves a partial install
that the next command misinterprets. No fault-injection tests exist.

## Strategy (chosen): Operation journal

Staged generations don't fit this surface — targets are heterogeneous files
scattered across per-vendor directories plus user-owned custom roots; there is
no single pointer to atomically swap. A durable operation journal gives
deterministic recovery with the ordering guarantee the issue demands.

## What Changes

- New `src/state/journal.rs`: an atomically-persisted operation journal at
  `<config_dir>/install-journal.json`. Every tracked mutation records its
  prior content (bytes or absent) in the journal **before** performing the
  write; the journal is deleted on successful completion.
- Auto-recovery: the next `install`/`sync` detects a stale journal and rolls
  back all applied mutations in reverse (restore prior bytes / remove created
  file), warning on stderr, before proceeding fresh.
- `ce-ai doctor` flags a present journal as a finding so a crashed run is
  diagnosed, not silently absorbed.
- `install.rs`/`sync.rs` mutation sites route through the journal helpers;
  `state.json` remains the final write of every command (ordering guarantee).
- Fault injection via `CE_AI_FAIL_AFTER_WRITES=<N>`: tracked write N+1 fails.
  Integration tests prove recovery for early and late cut points.

## Out of Scope

- Directory-tree rollback byte-for-byte: tree copies record whether the root
  pre-existed; rollback removes trees that were newly created and leaves
  pre-existing ones (doctor still flags the journal for manual review).
- Transactionality for uninstall/init-prj (follow-up if ever needed).

## Risks

| Risk | Mitigation |
| --- | --- |
| Journal itself corrupt | Treated as absent after a stderr warning; state stays last-written |
| Perf: prior-content capture doubles large writes | Managed files are small text assets; measured by existing benches |
