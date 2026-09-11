# Specification: Guaranteed Turn-0 Drift Delivery for OpenCode

## Requirements

### Requirement 1: Canonical OpenCode Plugin with Lifecycle Hooks
- **WHEN** an OpenCode session is initialized (`session.created`),
- **THEN** the plugin MUST execute `ce-ai workflow resume` in the session's workspace directory and inject the resulting output into the session via `client.session.prompt` with `{ noReply: true }`.
- **WHEN** an OpenCode session undergoes context compaction (`experimental.session.compacting`),
- **THEN** the plugin MUST execute `ce-ai workflow resume` and append the resulting output to `output.context`.

### Requirement 2: Embedded Canonical Loader in `ce-ai`
- **WHEN** `ce-ai install --harness opencode` or `ce-ai sync` runs,
- **THEN** `ce-ai` MUST install `.opencode/plugins/compound-engineering.js` containing the `session.created` hook into `<opencode_config_dir>/compound-engineering/plugins/compound-engineering.js` using `write_atomic`, regardless of whether the external source tarball contains the hook.

### Requirement 3: Idempotent Plugin Registration and Surgical Deinstallation
- **WHEN** `ensure_session_start_plugin` runs,
- **THEN** it MUST ensure the plugin is registered in `opencode.json` under `plugin[]` without duplicating existing entries, preserving any user-configured plugins.
- **WHEN** `ce-ai uninstall --harness opencode` runs,
- **THEN** it MUST remove the plugin entry from `opencode.json` without removing or mutating any user-configured custom plugins or custom skills.

### Requirement 4: Health Diagnostics in `ce-ai doctor`
- **WHEN** `ce-ai doctor` runs and `opencode` is recorded as an installed harness in `state.json`,
- **THEN** `ce-ai doctor` MUST verify that `<opencode_config_dir>/compound-engineering/plugins/compound-engineering.js` exists, contains `session.created`, and is registered in `opencode.json`.
- **WHEN** the plugin file is missing, does not contain `session.created`, or is missing from `opencode.json`,
- **THEN** `ce-ai doctor` MUST emit a finding prompting the user to run `ce-ai sync` or `ce-ai install --harness opencode`.
