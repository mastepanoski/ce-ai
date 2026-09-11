# Specification: Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI

## Requirements

### Requirement 1: Native Copilot CLI `sessionStart` Hook Management
- **WHEN** `ensure_session_start_hook` is called with a path to a Copilot hooks configuration file (e.g. `.github/hooks/hooks.json`),
- **THEN** it MUST ensure the file exists, has `"version": 1`, and contains a `sessionStart` hook command executing `ce-ai workflow resume --json` under both `bash` and `powershell` keys with atomic writes (`write_atomic`).
- **WHEN** the hooks file already contains user-defined hooks (e.g. `preToolUse`, other `sessionStart` commands),
- **THEN** `ensure_session_start_hook` MUST preserve all pre-existing hooks and settings without duplicating the `ce-ai` command.

### Requirement 2: Surgical Hook Deinstallation
- **WHEN** `remove_session_start_hook` is called on a project containing `.github/hooks/hooks.json`,
- **THEN** it MUST remove the `ce-ai workflow resume --json` entry from `sessionStart`.
- **WHEN** `hooks.json` becomes empty of all hooks and contains only `"version": 1`,
- **THEN** it MUST delete `hooks.json` and prune the parent `.github/hooks` directory if empty.

### Requirement 3: Context Payload Format
- **WHEN** `ce-ai workflow resume --json` is executed,
- **THEN** its JSON output MUST include an `additionalContext` string containing the formatted status text lines, enabling direct context ingestion by GitHub Copilot CLI and SDK consumers.

### Requirement 4: Health Diagnostics in `ce-ai doctor`
- **WHEN** an adopted project in `state.projects` contains a `.github` directory,
- **THEN** `ce-ai doctor` MUST verify that `.github/hooks/hooks.json` exists and contains the `sessionStart` hook.
- **WHEN** the hook is missing or unconfigured,
- **THEN** `ce-ai doctor` MUST emit a finding prompting the user to re-run `ce-ai init-prj`.
