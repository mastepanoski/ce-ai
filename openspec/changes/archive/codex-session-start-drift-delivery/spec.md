# Specification: Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI

## Requirements

### Requirement 1: Native Codex CLI `SessionStart` Hook Management
- **WHEN** `ensure_session_start_hook` is called with a path to a Codex TOML configuration file (e.g. `.codex/config.toml`),
- **THEN** it MUST ensure the file contains a `[[hooks.SessionStart]]` table with `matcher = "startup|resume|compact"` and a command hook running `ce-ai workflow resume`, serialized cleanly to TOML via atomic writes (`write_atomic`).
- **WHEN** the TOML file already contains user-defined hooks or other tables (e.g. `mcp_servers`, `PreToolUse`),
- **THEN** `ensure_session_start_hook` MUST preserve all pre-existing tables and settings without duplicating the `ce-ai` command.

### Requirement 2: Surgical Hook Deinstallation
- **WHEN** `remove_session_start_hook` is called on a project containing `.codex/config.toml`,
- **THEN** it MUST remove the `ce-ai workflow resume` entry from `SessionStart`.
- **WHEN** `config.toml` becomes empty of all keys as a result of hook removal,
- **THEN** it MUST delete `config.toml` and prune the parent `.codex` directory if empty.

### Requirement 3: Health Diagnostics in `ce-ai doctor`
- **WHEN** an adopted project in `state.projects` contains a `.codex` directory,
- **THEN** `ce-ai doctor` MUST verify that `.codex/config.toml` exists and contains the `SessionStart` hook.
- **WHEN** the hook is missing or unconfigured,
- **THEN** `ce-ai doctor` MUST emit a finding prompting the user to re-run `ce-ai init-prj`.
