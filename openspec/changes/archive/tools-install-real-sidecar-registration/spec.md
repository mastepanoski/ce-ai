# OpenSpec Requirements: Sidecar Installation Contracts

- **Change:** `tools-install-real-sidecar-registration`
- **Issue:** #158 (P0)

---

## 📜 Acceptance Criteria

### Requirement 1: Atomic Config Merging
- **WHEN** the user runs `ce-ai tools install <tool>`
- **THEN** `ce-ai` SHALL merge `<tool>` MCP server definition into `opencode.json` using `write_atomic`
- **AND** SHALL preserve all pre-existing user MCP servers and custom skills.

### Requirement 2: Mandatory Health Probe Verification
- **WHEN** configuration merge completes
- **THEN** `ce-ai` SHALL probe tool capability or binary presence
- **IF** the health probe fails
- **THEN** `ce-ai` SHALL return non-zero exit code (`CeError::Verification`) and SHALL NOT emit a success message.
