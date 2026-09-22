# Specification: Post-Merge OpenSpec Archival & Turn-0 Prescriptions

## Formal Requirements

### Requirement 1: Agent Governance Directive (Invariant #10)
- **WHEN** an AI agent inspects `AGENTS.md` or the managed adoption block in `AGENTS.md` / `CLAUDE.md`,
- **THEN** Invariant #10 MUST instruct the agent to run `ce-ai workflow status` post-merge and immediately archive any completed OpenSpec change packages via `ce-ai archive <feature>`.

### Requirement 2: Turn-0 Resume Prescription
- **WHEN** `ce-ai workflow resume` is executed in a repository where one or more change packages in `openspec/changes/` have all tasks checked in `tasks.md`,
- **THEN** the output MUST display a prominent action directive:
  `! Action Required: OpenSpec change '<feature>' is complete (<completed>/<total> tasks). Run 'ce-ai archive <feature>' to seal the change package.`

### Requirement 3: Doctor Actionable Guidance
- **WHEN** `ce-ai doctor` detects a completed but unarchived OpenSpec change package,
- **THEN** the warning message MUST explicitly include the command `run 'ce-ai archive <feature>'`.

### Requirement 4: Workflow Status Actionable Guidance
- **WHEN** `ce-ai workflow status` detects completed unarchived OpenSpec change packages,
- **THEN** the warning message MUST explicitly include `run 'ce-ai archive <feature>'`.
