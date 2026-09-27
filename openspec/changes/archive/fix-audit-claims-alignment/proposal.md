# Proposal: Align audited claims with delivered behavior

## Problem

The product audit found one destructive uninstall path and several user-facing
claims that describe stronger guarantees than the current commands provide.
Kimi backups are stored under the ambiguous filename `mcp.json`, so uninstall
cannot locate the original configuration and can delete a user-owned MCP entry
that shares a CE companion name. Documentation and help also state that resume
loads Engram memory, that `models set` creates a configuration backup, and that
FSM/gate commands enforce empirical verification; none is currently true.

## In scope

- Preserve and restore backups for all native harnesses whose configuration
  filenames are ambiguous, beginning with Kimi, AGY, and FX.
- Add hermetic regression coverage for the Kimi install/uninstall data-loss
  reproduction and backup discovery, and ensure AGY legacy cleanup runs even
  when a configuration snapshot is restored.
- Correct CLI/help and public documentation to state the actual boundaries of
  workflow resume, checkpoints, gates, and model assignment writes.
- Repair the stale Getting Started README anchor.

## Out of scope

- Adding an Engram client to the Rust binary.
- Turning workflow checkpoints or the gate into a test runner / CI enforcer.
- Changing the supported-harness set or remote installer behavior.

## Success criteria

1. A Kimi install followed by uninstall restores the byte-identical original
   `mcp.json`, including a pre-existing `codegraph` entry.
2. Backup listing/filtering recognizes Kimi, AGY, and FX snapshots, and AGY
   legacy cleanup is not skipped by snapshot restoration.
3. No public command help or user guide claims that `workflow resume` reads
   Engram memory, `models set` creates config backups, or gates verify tests.
4. Workflow documentation distinguishes legal checkpoint transitions from
   evidence-based verification.
