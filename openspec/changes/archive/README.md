# OpenSpec Change Archive

Completed change folders live here. A folder is archived when **either**:

1. **All tasks checked** — mechanical completion, no annotation needed; or
2. **STATUS-verified shipped** — `tasks.md` opens with a `> STATUS` header
   citing feature-level ship evidence (live code symbols or CHANGELOG
   release). Any residual open boxes under this criterion were **not**
   re-audited item-by-item and are declared unaudited by the header itself.

Folders must never be deleted: they are the audit trail linking shipped
releases to their frozen contracts.

## Triage — active folders with open tasks

| Folder | Open boxes | Next action |
| :--- | :--- | :--- |
| `../context_exhaustion_resilience/` | 15 | No observable ship evidence found in v1.20.0 sweep — needs feature-level verification before archive or revival |

Historical note (v1.20.1 sweep): 51 folders were archived in bulk under
criteria (1) and (2). The sweep's evidence sources: CHANGELOG cross-references
and live code symbols (`BLOCK_VERSION`, doctor probes #112, `SkillRegistry`,
installer CI gates, worktree probes, exit-code contract).
