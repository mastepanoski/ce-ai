# Requirements Specification: Claude MD Delegation & Deduplication

## Scenarios & Acceptance Criteria

### Scenario 1: Fresh project adoption with `.claude/` present
- **GIVEN** a project directory with `.claude/` (e.g. `.claude/settings.json`) and no `CLAUDE.md` or `AGENTS.md`
- **WHEN** `ce-ai init-prj` is executed
- **THEN** `AGENTS.md` is created containing the managed 7-stage block
- **AND** `CLAUDE.md` is created containing solely `@AGENTS.md\n`
- **AND** `CLAUDE.md` does NOT contain `<!-- CE-AI MANAGED BLOCK BEGIN -->` or duplicate managed instructions
- **AND** `.claude/settings.json` receives the expected Claude hooks.

### Scenario 2: Project with `CLAUDE.md` importing `@AGENTS.md` and user directives
- **GIVEN** a project directory where `AGENTS.md` exists and `CLAUDE.md` contains `@AGENTS.md\n\n# User Instructions\nDo not push to main.`
- **WHEN** `ce-ai init-prj` or `ce-ai sync` is executed
- **THEN** `CLAUDE.md` is NOT modified with a duplicate managed block
- **AND** user instructions remain intact.

### Scenario 3: Healing existing duplicate block in `CLAUDE.md`
- **GIVEN** a project where `AGENTS.md` exists and `CLAUDE.md` contains:
  ```markdown
  @AGENTS.md

  <!-- CE-AI MANAGED BLOCK BEGIN -->
  ## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement
  ...
  <!-- CE-AI MANAGED BLOCK END -->
  ```
- **WHEN** `ce-ai sync` or `ce-ai init-prj` is executed
- **THEN** the duplicate `CE_MANAGED_BEGIN...CE_MANAGED_END` block is stripped from `CLAUDE.md`
- **AND** `CLAUDE.md` is restored to `@AGENTS.md\n`.

### Scenario 4: Project without `AGENTS.md` delegation
- **GIVEN** a project where `CLAUDE.md` contains custom rules without any `@AGENTS.md` directive
- **WHEN** `ce-ai init-prj` or `ce-ai sync` is executed
- **THEN** `CLAUDE.md` receives the managed block via `update_claude_md`, ensuring Claude Code gets the rules.

### Scenario 5: De-adoption cleanup
- **GIVEN** an adopted project where `CLAUDE.md` contains `@AGENTS.md`
- **WHEN** `ce-ai deinit-prj` is executed
- **THEN** `CLAUDE.md` is cleanly removed if it only contained the stub.
