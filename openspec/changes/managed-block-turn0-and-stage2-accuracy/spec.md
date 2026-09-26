# Specification: Managed Adoption Block Turn-0 Directives & Progressive OpenSpec Accuracy

## Requirement 1: Conditional Turn-0 Session Directives
- **WHEN** `render_block_content(AdoptionTier::Full)` is called,
- **THEN** the returned text MUST contain the conditional Turn-0 directive:
  - Mentioning that `ce-ai` auto-installs a SessionStart hook running `ce-ai workflow resume --json`.
  - Directing the agent that if state is already present at session start, treat it as current and DO NOT re-run the command.
  - Instructing agents to only run `ce-ai workflow resume` when context was not injected or when suspicion of mid-session stale state arises.

## Requirement 2: Progressive OpenSpec Requirements
- **WHEN** `render_block_content(AdoptionTier::Full)` is called,
- **THEN** the returned text MUST describe progressive OpenSpec authoring:
  - Specifying that before writing feature code, agents verify the frozen Stage 2 contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`).
  - Specifying that `tasks.md` is generated in Stage 3 from that frozen contract, and must not be required before Stage 2 is complete, but must be present before opening a PR.

## Requirement 3: BLOCK_VERSION Bump
- **WHEN** `init_prj` adopts or upgrades a project at Full tier,
- **THEN** the rendered block header MUST declare `v=7`.
- **WHEN** `state.json` records adoption metadata,
- **THEN** `block_version` MUST be recorded as `7`.

## Requirement 4: Stale Version Upgrade Compatibility
- **WHEN** an adopted project has an existing block with `v=6`,
- **THEN** `ce-ai doctor` and `ce-ai status` MUST classify it as `StaleVersion { version: 6 }` and advise running `ce-ai init-prj --tier full to upgrade`.
- **WHEN** `ce-ai init-prj --tier full` is rerun on that project,
- **THEN** it MUST cleanly upgrade the block to `v=7`, preserve non-managed content, and record `block_version = 7` in `state.json`.

## Requirement 5: Non-Regression of Other Tiers
- **WHEN** `render_block_content` is called for `AdoptionTier::Minimal` or `AdoptionTier::Orchestrator`,
- **THEN** the returned content MUST remain byte-for-byte identical to their previous definitions.
