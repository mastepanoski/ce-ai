# Spec Delta: `sync-native-registration-fix`

## Requirement: Native harness re-registration during sync

### R1 Explicit re-registration

- **WHEN** `ce-ai sync` rebuilds a state entry for `kimi`, `agy`, or `fx`,
  **THEN** the CLI MUST register the `codegraph` and `engram` MCP servers
  through that vendor's native config writer and copy the managed skills
  tree into the vendor root, exactly as `install` does.
- **WHEN** `ce-ai sync` rebuilds a state entry for `pi`,
  **THEN** the CLI MUST copy the managed skills tree into `<root>/skills`
  and MUST NOT write any JSON configuration file.
- **WHEN** a harness name without an explicit registration arm is found in
  `state.json`, **THEN** the CLI MUST fail with `CeError::Runtime` naming
  the entry instead of silently applying OpenCode-format mutations.
- **WHEN** a managed-skills copy fails for any harness arm,
  **THEN** the CLI MUST surface the IO error rather than swallow it.

### R2 Native config preservation

- **WHEN** `ce-ai sync` processes kimi/agy/fx entries whose native config
  files exist,
  **THEN** the files MUST remain byte-identical apart from the vendor
  writer's own managed server entries; OpenCode-format keys
  (`plugin`, `skills.paths`) MUST NOT appear.

### R3 Verification coverage

- **WHEN** the post-sync verification matrix runs for
  claude/codex/copilot/grok/kimi/agy/pi/fx,
  **THEN** each surface MUST be SHA256-hash-checked against its per-kind
  skills root (agy: `<root>/config/skills`; others: `<root>/skills`) and
  drift MUST fail sync with exit code 6.
