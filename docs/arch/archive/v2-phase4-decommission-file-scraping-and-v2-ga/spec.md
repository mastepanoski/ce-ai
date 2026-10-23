# Specification: Phase 4 — Full Decommissioning of File Scraping & CE-AI v2.0 Release (v2.0.0 GA)

## Requirements & Acceptance Criteria

### REQ-1: Deprecation Warnings on Legacy File Scraping Surfaces
- **WHEN** a user or agent invokes `ce-ai install`,
- **THEN** `ce-ai` MUST emit a clear deprecation advisory recommending `ce-ai init-prj` and `ce-ai fleet pin / sync`.
- **WHEN** a user or agent invokes `ce-ai upgrade`,
- **THEN** `ce-ai` MUST emit a deprecation advisory directing them to `ce-ai fleet pin / sync`.
- **WHEN** a user or agent invokes `ce-ai sync`,
- **THEN** `ce-ai` MUST emit a deprecation advisory highlighting that native host package managers are driven via `ce-ai fleet sync`.

### REQ-2: Neutralization of Watch Loop
- **WHEN** `ce-ai sync --watch` is invoked,
- **THEN** `ce-ai` MUST output a deprecation warning clarifying that continuous file restoration is deprecated in v2 in favor of native host lifecycle governance.

### REQ-3: Preserved Binary Self-Update Security
- **WHEN** `ce-ai self-update` is executed,
- **THEN** `ce-ai` MUST continue to extract the self-update binary release securely using directory traversal checks (`is_safe_relative_path`).

### REQ-4: v2.0 Identity & Documentation Conformance
- **WHEN** inspecting `README.md`,
- **THEN** it MUST stay ≤ 100 lines and define `ce-ai` as the *Cross-Host Operational Companion for Compound Engineering*.
- **WHEN** inspecting `Cargo.toml`,
- **THEN** the package version MUST be `2.0.0` and its description MUST reflect the cross-host operational companion identity.
