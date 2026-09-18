# OpenSpec Change Archive

Completed change folders live here. A folder is archived when **either**:

1. **All tasks checked** — mechanical completion, no annotation needed; or
2. **STATUS-verified shipped** — `tasks.md` opens with a `> STATUS` header
   citing feature-level ship evidence (live code symbols or CHANGELOG
   release). Any residual open boxes under this criterion were **not**
   re-audited item-by-item and are declared unaudited by the header itself.

Folders must never be deleted: they are the audit trail linking shipped
releases to their frozen contracts.

## Compacted Milestones

| Milestone | Changes Compacted | Rollup Summary | Raw Archive |
| :--- | :--- | :--- | :--- |
| 2026-Q3 | 110 | [milestones/2026-Q3.md](milestones/2026-Q3.md) | [milestones/archive-2026-Q3.tar.gz](milestones/archive-2026-Q3.tar.gz) |

## Triage — active folders with open tasks

| Folder | Open boxes | Next action |
| :--- | :--- | :--- |
| *(none — ledger clean as of v1.44.2)* | | |

Historical notes:
- v1.20.1 sweep: 51 folders archived under criteria (1) and (2); evidence
  sources were CHANGELOG cross-references and live code symbols
  (`BLOCK_VERSION`, doctor probes #112, `SkillRegistry`, installer CI gates,
  worktree probes, exit-code contract).
- v1.21.0: `context_exhaustion_resilience` completed its last open
  requirement (doctor branch-protection probe) and was archived fully
  checked.
- v1.44.2 sweep: 7 folders archived under criterion (2) (STATUS-verified
  shipped); evidence sources were live code symbols across harness hooks
  (Codex, Pi, Cursor), generic session-start drift delivery, CodeGraph init
  commands/tests, and pedagogical guardrail commands.
- 2026-09-11 sweep: 33 folders archived via 'ce-ai archive --all'.
- openspec-archive-command: archived (15/15 tasks) under criterion (1) on 2026-09-11.
- blocking-gate-check: archived (24/24 tasks) under criterion (1) on 2026-09-12.
- doc-debt-engine-and-probes: archived (31/31 tasks) under criterion (1) on 2026-09-12.
- archive-compaction: archived (33/33 tasks) under criterion (1) on 2026-09-13.
- 2026-09-13 sweep: 3 folders archived via 'ce-ai archive --all'.
- solution-library-lint-and-repair: archived (19/19 tasks) under criterion (1) on 2026-09-15.
- living-system-specs-promotion: archived (31/31 tasks) under criterion (1) on 2026-09-15.
- solution-refresh-and-deduplication: archived (20/20 tasks) under criterion (1) on 2026-09-15.
- fix-claude-agents-block-dedup: archived (17/17 tasks) under criterion (1) on 2026-09-18.
- fix-companion-mcp-args: archived (26/26 tasks) under criterion (1) on 2026-09-18.
- odd-mode-router-and-graduation: archived (45/45 tasks) under criterion (1) on 2026-09-18.
- model-routing: archived (27/27 tasks) under criterion (1) on 2026-09-18.
- skill-routing: archived (25/25 tasks) under criterion (1) on 2026-09-18.
- risk-aware-execution: archived (25/25 tasks) under criterion (1) on 2026-09-18.
