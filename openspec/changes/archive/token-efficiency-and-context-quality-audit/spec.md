# OpenSpec Requirements: Behavior Rules & Acceptance Criteria

- **Change:** `token-efficiency-and-context-quality-audit`
- **Issue:** #117

---

## 📜 Acceptance Criteria

### Requirement 1: Multi-Harness & Capability-Based Detection
- **WHEN** the user runs `ce-ai audit`
- **THEN** `ce-ai` SHALL detect all installed harnesses (`opencode`, `claude`, etc.)
- **AND** SHALL evaluate capability detectors (`cli-output-compression`, `mcp-sprawl`, `prompt-duplication`, `persistent-memory`, `docs-grounding`, `code-intelligence`, `learnings-library`)
- **AND** SHALL output satisfied attribution (`satisfied-by: <tool>`).

### Requirement 2: Advisory Score Calculation
- **WHEN** audit execution completes
- **THEN** `ce-ai` SHALL compute score percentage `(PASS*1.0 + WARN*0.5) / TotalApplicable * 100`
- **AND** SHALL exit with code 0 by default.

### Requirement 3: JSON Output
- **WHEN** the user passes `--json`
- **THEN** `ce-ai` SHALL output formatted JSON matching `AuditReport` schema.

### Requirement 4: `--fail-under` Threshold Enforcement
- **WHEN** the user passes `--fail-under <pct>`
- **AND** score percentage is strictly less than `<pct>`
- **THEN** `ce-ai audit` SHALL exit with non-zero exit code (Exit 1).
