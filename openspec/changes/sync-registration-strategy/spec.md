# Spec Delta: `sync-registration-strategy`

## Requirement: Exhaustive re-registration table

- **WHEN** a new `HarnessKind` variant is added,
  **THEN** `registration_spec` MUST fail to compile until the variant is
  classified in the table or given a dedicated call-site arm.
- **WHEN** `ce-ai sync` rebuilds any of cursor/claude/codex/copilot/grok/
  kimi/agy/fx,
  **THEN** behavior MUST remain identical to PR #206: vendor MCP
  registration for codegraph and engram (except pi), managed-skills recopy
  into the spec's subpath, propagated errors.
- **WHEN** opencode, custom, or deepseek entries are processed,
  **THEN** they MUST keep their dedicated arms (JSON writer, snapshot flow,
  named Runtime error respectively).
