# OpenSpec Requirements: Behavior Rules

- **Change:** `adoption-block-staleness-alignment`
- **Issue:** #149

---

## 📜 Acceptance Criteria

### Requirement 1: Shared Classification Helper
- **WHEN** checking adoption block status for a project
- **THEN** both `ce-ai doctor` and `ce-ai status` SHALL invoke `check_adoption_block_status`.

### Requirement 2: Actionable Upgrade Hint in `status`
- **WHEN** an adopted project's block version is older than `BLOCK_VERSION` (`v < BLOCK_VERSION`)
- **THEN** `ce-ai status` SHALL output `STALE BLOCK v=<version> — re-run ce-ai init-prj --tier <tier> to upgrade`.
