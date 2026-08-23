# Adoption Block Staleness Alignment Requirements

- **Date:** 2026-08-22
- **Issue:** #149 (status: align adoption-block staleness wording with doctor's actionable hint)
- **Status:** Approved
- **Scope Tier:** Lightweight / Standard Alignment

---

## 🎯 1. Problem Statement

`ce-ai doctor` distinguishes between:
- **Stale adoption block version** (`v < BLOCK_VERSION`): `project-adoption: stale block version v=1 at '<path>' — re-run ce-ai init-prj --tier full to upgrade`
- **Generic SHA drift**: `project-adoption: block SHA drift detected at '<path>'`

However, `ce-ai status` reports only generic `status: DRIFT DETECTED` for both cases. This forces operators to run `doctor` just to discover the exact actionable upgrade step.

---

## 🚀 2. Goals & Success Criteria

1. **Shared Classification Helper**: Extract adoption block status parsing and version extraction into `crate::commands::init_prj::check_adoption_block_status`.
2. **Actionable Status Output**: `ce-ai status` surfaces `STALE BLOCK v=1 — re-run ce-ai init-prj --tier <tier> to upgrade` when a block is on an older version.
3. **100% Diagnostic Parity**: Both `ce-ai doctor` and `ce-ai status` share the same single source of truth for adoption block status classification.
