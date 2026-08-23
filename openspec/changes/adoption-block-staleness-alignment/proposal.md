# OpenSpec Proposal: Adoption Block Staleness Alignment

- **Change:** `adoption-block-staleness-alignment`
- **Issue:** #149
- **Author:** Antigravity AI
- **Date:** 2026-08-22

---

## 🎯 1. Problem & Context
`ce-ai status` outputs generic `DRIFT DETECTED` for adopted projects even when the on-disk block is simply an older template version (`v < BLOCK_VERSION`). In contrast, `ce-ai doctor` provides a targeted upgrade command (`re-run ce-ai init-prj --tier <tier> to upgrade`).

## 🚀 2. Proposed Solution
Refactor block status classification into `crate::commands::init_prj::check_adoption_block_status` so both `doctor` and `status` consume a shared helper and present consistent, actionable upgrade hints.
