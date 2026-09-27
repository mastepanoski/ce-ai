---
title: "Native Harness Backup Identity Prevents Configuration Loss"
date: 2026-09-27
category: bugfixes
module: state/backups
problem_type: data_loss_prevention
component: harness-uninstall
severity: high
tags:
  - backups
  - uninstall
  - kimi
  - agy
  - fx
  - configuration-safety
applies_when: "Adding or changing harness configuration backups whose native filenames are shared across harnesses."
---

# Native Harness Backup Identity Prevents Configuration Loss

## Context

Kimi and FX both use `mcp.json`; AGY uses `mcp_config.json`. Backup discovery
filters snapshots by a harness identity inferred from the persisted filename.
A bare native filename therefore lost the information needed to select the
matching snapshot during uninstall.

For Kimi, this could cause uninstall to skip the original snapshot and remove
CE companion names from the live configuration. A user-owned `codegraph` entry
with the same name could be lost.

## Resolution

Create native snapshots with an explicit `kimi-`, `agy-`, or `fx-` filename
prefix. The existing backup filter can then select the right snapshot and
restore its original bytes before fallback name-based cleanup is considered.

Keep cleanup unrelated to the MCP fallback outside the restore-or-fallback
branch. AGY's legacy `antigravity.json` is one such artifact: it must be
removed whether uninstall restored a configuration snapshot or unregistered
individual MCP entries.

## Verification

- Unit coverage confirms identity-aware backup discovery and restoration for
  Kimi, AGY, and FX.
- CLI coverage confirms a Kimi user-owned `codegraph` entry is restored
  byte-for-byte after install and uninstall.
- CLI coverage confirms AGY removes its legacy artifact after snapshot
  restoration.

## Apply This Pattern

When a backup lookup relies on a derived identifier, store that identifier in
the snapshot metadata or filename. Do not infer ownership from a filename that
multiple producers can share.
