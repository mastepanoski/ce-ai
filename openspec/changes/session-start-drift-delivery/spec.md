# Specification: Guaranteed Turn-0 Drift Delivery

## Requirements

### Requirement 1: Claude Code SessionStart Hook Injection
- **WHEN** `ce-ai init-prj` is executed on a project containing a `.claude/` directory,
- **THEN** it MUST ensure `.claude/settings.json` contains a `SessionStart` hook matching `.*` with command `ce-ai workflow resume`.
- **WHEN** `.claude/settings.json` already contains user hooks or configuration keys,
- **THEN** it MUST preserve all existing keys and append/ensure the `ce-ai workflow resume` hook without overwriting user settings.
- **WHEN** `ce-ai deinit-prj` is executed on the project,
- **THEN** it MUST surgically remove the `ce-ai workflow resume` hook from `.claude/settings.json`.

### Requirement 2: Universal Turn-0 Textual Directive & Block Version 4
- **WHEN** `render_block_content(AdoptionTier::Full)` is rendered,
- **THEN** it MUST contain explicit Turn-0 directives mandating `ce-ai workflow resume` at session start.
- **WHEN** `BLOCK_VERSION` is queried,
- **THEN** it MUST evaluate to `4`.
- **WHEN** `ce-ai doctor` or `ce-ai status` inspects an adopted project with `v < 4`,
- **THEN** it MUST report `StaleVersion` with an actionable upgrade recommendation.

### Requirement 3: Checkpoint Drift Warning Gate
- **WHEN** `ce-ai workflow checkpoint <stage> <task>` is executed,
- **THEN** it MUST probe the live repository state and append a warning if `manifest_drift_count > 0`.

### Requirement 4: Doctor Claude Hook Audit
- **WHEN** `ce-ai doctor` is executed and an adopted project has a `.claude/` directory lacking the `SessionStart` hook,
- **THEN** it MUST emit a `claude-hook-missing` finding pointing to `.claude/settings.json`.

### Requirement 5: Documentation Honesty Alignment
- **WHEN** documentation is consulted in `zero-step-drift-recovery-explained.md`, `harnesses-loops-and-context-masterclass.md`, or `workflow-panel-native-vs-agent-skills.md`,
- **THEN** it MUST distinguish between automated hook execution (Claude Code) and enforced prompt directives (other harnesses), and accurately describe `workflow resume` capabilities without placeholder disclaimers.
