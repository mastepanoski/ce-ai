# OpenSpec Requirements: Behavior Rules & Acceptance Criteria

- **Change:** `companion-tool-readiness-and-freshness`
- **Issue:** #112
- **Author:** Antigravity AI
- **Date:** 2026-08-22

---

## 📜 Specifications & Acceptance Criteria

### Requirement 1: Version Freshness Probes in `tools status`
- **WHEN** the user runs `ce-ai tools status`
- **THEN** `ce-ai` SHALL detect the installed version of Engram, CodeGraph, Context7, RTK, and `ce-ai` itself
- **AND** SHALL compare installed versions against the expected registry version
- **AND** SHALL display status tags `ok`, `outdated`, `missing`, or `unknown (offline)`.

### Requirement 2: Actionable Remediation Hints
- **WHEN** a companion tool or skill is reported as `outdated` or `missing`
- **THEN** `ce-ai` SHALL print the exact suggested remediation command (e.g. `ce-ai tools install codegraph`, `ce-ai upgrade`, or `ce-ai skills resolve sequential-thinking`).

### Requirement 3: Doctor Exit Code Policy & `--strict` Flag
- **WHEN** the user runs `ce-ai doctor` without `--strict`
- **THEN** missing tools SHALL push a finding (failing doctor with non-zero exit code)
- **AND** outdated tools SHALL print informational hints (`doctor-info: ...`) WITHOUT failing doctor (Exit 0).
- **WHEN** the user runs `ce-ai doctor --strict`
- **THEN** both missing AND outdated tools SHALL push a finding (failing doctor with non-zero exit code).

### Requirement 4: Offline Degradation Resilience
- **WHEN** the environment is offline or the network request to refresh registry cache fails or times out (~500ms)
- **THEN** version checks SHALL degrade gracefully to local cache or embedded defaults
- **AND** SHALL report `(offline)` WITHOUT raising network errors or failing exit codes.

### Requirement 5: Non-Destructive Config Merging
- **WHEN** companion tools or MCP sidecars are wired into harness configurations (`opencode.json`, `claude.json`)
- **THEN** unmanaged user plugins, custom skills, and custom MCP servers SHALL be preserved 100% untouched.

### Requirement 6: Self-Update Notification
- **WHEN** `ce-ai` detects that a newer release is available on GitHub
- **THEN** `ce-ai doctor` SHALL print an informational hint naming `ce-ai upgrade` as the upgrade path.
