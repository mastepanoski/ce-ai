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
| *(none — ledger clean as of v1.21.0)* | | |

Historical notes:
- v1.20.1 sweep: 51 folders archived under criteria (1) and (2); evidence
  sources were CHANGELOG cross-references and live code symbols
  (`BLOCK_VERSION`, doctor probes #112, `SkillRegistry`, installer CI gates,
  worktree probes, exit-code contract).
- v1.21.0: `context_exhaustion_resilience` completed its last open
  requirement (doctor branch-protection probe) and was archived fully
  checked.
