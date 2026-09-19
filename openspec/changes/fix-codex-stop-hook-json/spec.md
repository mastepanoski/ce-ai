# Specification: Codex and Multi-Harness Stop Hook Clean JSON Output

## Behavioral Requirements

### Requirement 1: Stop Hook Clean JSON Output
- **WHEN** `ce-ai workflow resume` is executed with `--event Stop` (or `--event stop`),
- **THEN** it SHALL execute `maybe_auto_checkpoint`,
- **AND** it SHALL emit `{}` to `stdout` without printing human-readable text lines,
- **AND** it SHALL exit with code 0.

### Requirement 2: Dynamic Stdin Stop Hook Detection
- **WHEN** `ce-ai workflow resume` is executed without `--event` in a non-terminal environment,
- **AND** `stdin` contains a JSON payload with `"hook_event_name": "Stop"` (or `"stop_hook_active"`),
- **THEN** it SHALL automatically identify the execution as a `Stop` hook,
- **AND** it SHALL execute `maybe_auto_checkpoint`,
- **AND** it SHALL emit `{}` to `stdout`,
- **AND** it SHALL exit with code 0.

### Requirement 3: Stop Hook Loop Prevention
- **WHEN** `stdin` contains a JSON payload with `"stop_hook_active": true`,
- **THEN** it SHALL emit `{}` to `stdout`,
- **AND** it SHALL exit with code 0 without triggering additional checkpoint side-effects.

### Requirement 4: PreCompact Hook Clean JSON Output
- **WHEN** `ce-ai workflow resume` is executed with `--event PreCompact` (or detected via `stdin` `"hook_event_name": "PreCompact"`),
- **THEN** it SHALL execute `maybe_auto_checkpoint`,
- **AND** it SHALL emit `{}` to `stdout`,
- **AND** it SHALL exit with code 0.

### Requirement 5: Interactive Terminal Experience Preservation
- **WHEN** `ce-ai workflow resume` is executed from an interactive terminal (`std::io::stdin().is_terminal() == true`),
- **AND** no `--event` or `--json` flags are provided,
- **THEN** it SHALL output standard human-readable workflow resume lines.

### Requirement 6: Codex and Claude Adapter Resiliency
- **WHEN** `has_session_start_hook` or `remove_session_start_hook` is called in `src/harness/codex.rs` or `src/harness/claude.rs`,
- **THEN** it SHALL correctly detect and manage commands matching `ce-ai workflow resume` with or without extra flags.
