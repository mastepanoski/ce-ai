# Design

## Backup identity

`state::backups::backup_file` will derive a stable, harness-prefixed filename
from the source path for Kimi (`kimi-`), AGY (`agy-`), and FX (`fx-`) in the
same way it already does for Cursor, Claude, Codex, Copilot, and Grok.
`harness_from_path` already recognizes these prefixes. Existing backups remain
readable as `custom`; the fix guarantees correct identity for newly created
backups. Install and uninstall remain atomic through existing `write_atomic`
calls.

## Regression tests

- Unit-test source-path-to-backup naming and filtered backup discovery for
  Kimi, AGY, and FX.
- Add a hermetic command-level Kimi install/uninstall test with a user-owned
  `codegraph` MCP entry, asserting exact original bytes are restored.
- Keep AGY legacy artifact cleanup outside the backup/fallback branch so a
  successful snapshot restoration cannot bypass it.

## Truthful surface

- Change resume help to refer to checkpoint/OpenSpec state rather than Engram.
- Rewrite affected explanatory text to describe live local probes.
- State that checkpoints validate declared legal transitions, while tests and
  knowledge capture are workflow obligations evidenced outside the command.
- State that `models set` uses atomic config writes and profile snapshots, not
  a pre-mutation configuration backup.
- Describe `gate check` as an OpenSpec-presence gate and LOC advisory.
