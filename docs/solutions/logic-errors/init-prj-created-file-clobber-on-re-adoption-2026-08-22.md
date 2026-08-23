---
title: "init-prj re-adoption clobbered created_file, breaking deinit-prj cleanup"
date: "2026-08-22"
category: logic-errors
module: commands/init_prj
problem_type: logic_error
component: tooling
severity: medium
symptoms:
  - "deinit-prj leaves orphaned AGENTS.md / CLAUDE.md in projects where ce-ai originally created them"
  - "state.json silently flips created_file from true to false after running init-prj twice (e.g. the recommended upgrade flow)"
root_cause: logic_error
resolution_type: code_fix
related_components:
  - documentation
  - development_workflow
tags:
  - init-prj
  - deinit-prj
  - state-json
  - idempotency
  - project-adoption
---

# init-prj re-adoption clobbered created_file, breaking deinit-prj cleanup

## Problem

`ce-ai init-prj` upgrade re-runs silently corrupted adoption state. When re-run over an existing adoption, the command rebuilt the entire `ProjectAdoptionEntry` from scratch with `created_file = !file_existed`. Because `AGENTS.md` already existed on disk at that point, the flag evaluated to `false` even when ce-ai had originally created the file. Since `deinit-prj` gates deletion of agent-created files on this flag, the documented upgrade→deinit flow left orphaned `AGENTS.md`/`CLAUDE.md` behind.

## Symptoms

- After following the CHANGELOG-recommended upgrade flow (`init-prj` again to pick up a new block version, later `deinit-prj`), `AGENTS.md`/`CLAUDE.md` remained in the project directory.
- The `created_file` flag in `state.json` flipped from `true` to `false` with no warning or diff surfaced to the user.
- No error was raised anywhere; the corruption was purely silent state drift.

## Why It Shipped

The defect shipped unnoticed because existing integration tests covered fresh adoption and plain de-init, but never the **re-run (upgrade) path**. The lesson: any code path that *replaces* a persisted record must be tested for field preservation, not just for success.

## Solution

On the replacement path in `init_prj`, carry the original `created_file` value forward instead of recomputing it (`src/commands/init_prj.rs`):

```rust
match state.projects.iter().position(|p| p.path == target_dir) {
    Some(pos) => {
        // Preserve who originally created the file: an upgrade
        // re-run replaces the entry, and deinit-prj relies on this
        // flag to clean up agent-created AGENTS.md/CLAUDE.md.
        entry.created_file = state.projects[pos].created_file;
        state.projects[pos] = entry;
    }
    None => {
        state.projects.push(entry);
    }
}
```

Plus a full lifecycle regression test, `init_prj_upgrade_rerun_preserves_created_file_flag` (`tests/cli.rs`): fresh adopt (flag `true`) → stale managed block written by hand → re-run `init-prj` (replacement path taken) → assert flag is still `true` → run `deinit-prj` → assert both files removed.

## Why This Works

`created_file` records **provenance** ("did ce-ai create this file?"), not current filesystem state. Provenance is immutable across the file's lifetime — re-running an upgrade changes the content hash and block version, never who created the file. Recomputing it from a filesystem probe conflates two different facts. Preserving the prior value turns the replacement path into a partial update: content/version fields refresh; ownership fields are carried forward. That is exactly the semantics `deinit-prj`'s delete gate assumes.

## Prevention

- Whenever a persisted entry is replaced wholesale, audit each field: which are "current state" (safe to recompute) versus "historical fact" (must be preserved)?
- Test every state-mutating command through its **full lifecycle**, including the idempotent/re-run path — not just happy-path first use.
- Keep explanatory comments on non-obvious field preservation so future refactors do not "simplify" it away.
- Cross-check upgrade flows against every consumer of the fields being rebuilt (`deinit-prj` read `created_file`; the drift probes read `block_sha256`).

## Related Issues

- [Project Adoption Engine: Non-Destructive Multi-Harness Governance](../architecture/project-adoption-engine-init-and-deinit-prj.md) — canonical design record this fix extends (updated with v2/BLOCK_VERSION details).
- Shipped in PR #140 / release v1.5.0.
