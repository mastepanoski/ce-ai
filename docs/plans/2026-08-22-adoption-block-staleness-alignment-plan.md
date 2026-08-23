# Implementation Plan: Adoption Block Staleness Alignment

- **Date:** 2026-08-22
- **Issue:** #149
- **Origin:** `docs/brainstorms/2026-08-22-adoption-block-staleness-alignment-requirements.md`
- **OpenSpec Change:** `adoption-block-staleness-alignment`

---

## 🎯 Summary
Extract adoption block classification into a shared helper in `src/commands/init_prj.rs` so that `ce-ai status` and `ce-ai doctor` share a single source of truth for block staleness and surface actionable upgrade hints.
