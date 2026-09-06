# Specification: Auto-configure RTK Hook Injection for Natively-Supported Harnesses

## Requirements & Acceptance Criteria

### Requirement 1: RTK Support Matrix
- **WHEN** `is_rtk_supported(kind)` is evaluated,
- **THEN** it MUST return `true` exclusively for `HarnessKind::Claude`, `HarnessKind::Cursor`, `HarnessKind::Copilot`, and `HarnessKind::Codex`.
- **THEN** it MUST return `false` for all other `HarnessKind` variants (`Opencode`, `Pi`, `Custom`, `Deepseek`, `Grok`, `Kimi`, `Agy`, `Fx`).

### Requirement 2: Explicit Opt-Out Mechanism
- **WHEN** `is_rtk_opted_out` is checked with either `skip_rtk = true` or `skip_companions = true`,
- **THEN** it MUST return `true`.
- **WHEN** the environment variable `CE_AI_SKIP_RTK` is set to `"1"`, `"true"`, or `"yes"`,
- **THEN** it MUST return `true` regardless of CLI flag values.
- **WHEN** the environment variable `CE_AI_SKIP_COMPANIONS` is set to `"1"`, `"true"`, or `"yes"`,
- **THEN** it MUST return `true` regardless of CLI flag values.
- **WHEN** no opt-out flags are passed and environment variables are unset/empty/`"0"`,
- **THEN** it MUST return `false`.

### Requirement 3: Install-Time Hook Configuration
- **WHEN** `ce-ai install --harness <supported>` is executed,
- **THEN** if RTK is available and not opted out, `ce-ai` MUST configure the RTK hook for that harness (`rtk init -g --auto-patch --agent claude`, `cursor`, `--copilot`, or `--codex`).
- **WHEN** `ce-ai install --harness <unsupported>` is executed,
- **THEN** `ce-ai` MUST treat RTK configuration as an explicit no-op and log an informational message when not quiet.
- **WHEN** `rtk` is not found on `PATH` during `install`,
- **THEN** `ce-ai` MUST print a warning diagnostic and MUST NOT fail the installation (`Ok(())`).
- **WHEN** `--dry-run` is passed to `ce-ai install`,
- **THEN** `ce-ai` MUST NOT execute `rtk init` and MUST output a dry-run plan.

### Requirement 4: Project Adoption Hook Configuration
- **WHEN** `ce-ai init-prj` is run on a project repository,
- **THEN** `ce-ai` MUST detect any supported harnesses present and configure RTK hooks for them, unless opted out.
- **WHEN** `--skip-rtk` or `--skip-companions` is passed to `ce-ai init-prj`,
- **THEN** RTK hook configuration MUST be skipped.

### Requirement 5: Symmetrical Uninstallation
- **WHEN** `ce-ai uninstall --harness <supported>` is executed,
- **THEN** `ce-ai` MUST run the corresponding RTK uninstall command to cleanly remove injected hooks.
- **WHEN** `ce-ai uninstall --harness <unsupported>` is executed,
- **THEN** `ce-ai` MUST treat RTK uninstallation as a silent no-op.

### Requirement 6: Audit Severity Escalation
- **WHEN** `ce-ai audit` evaluates `CliCompressionDetector`,
- **THEN** any supported harness lacking the RTK binary or hook MUST report `AuditStatus::Warn`.
- **THEN** any supported harness with an active RTK hook MUST report `AuditStatus::Pass`.
- **THEN** any unsupported harness MUST report `AuditStatus::Info` with a clear explanation that RTK hook injection is not supported for that harness.

### Requirement 7: Doctor Diagnostics & Known Limitation Disclosure
- **WHEN** `ce-ai doctor` runs,
- **THEN** it MUST check whether installed supported harnesses have RTK hooks configured and report warnings if missing.
- **THEN** it MUST output an advisory documenting the known limitation where RTK command filters may alter or swallow stdout on wrapped commands (e.g. `gh issue view --comments`).
