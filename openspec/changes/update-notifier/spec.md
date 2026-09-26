# Spec: Automated Background Update Notifier Requirements

## Functional Requirements

### REQ-1: Throttled Cache Expiration
- **WHEN** `ce-ai` executes and `<config_dir>/cache/update_check.json` is missing or `last_checked_at` is older than `interval_hours` (default 24 hours),
- **THEN** an update check MUST be triggered.
- **WHEN** `<config_dir>/cache/update_check.json` exists and was written less than `interval_hours` ago,
- **THEN** no network request SHALL be initiated.

### REQ-2: Non-Blocking Background Thread Execution
- **WHEN** an update check is triggered during regular CLI command execution,
- **THEN** the check MUST execute in a detached background thread without delaying subcommand dispatch.
- **WHEN** the background thread encounters network failure, HTTP 429/403 rate limits, timeout, or DNS resolution errors,
- **THEN** it MUST terminate silently without raising errors or altering exit codes.

### REQ-3: Stderr Banner Notification
- **WHEN** a CLI command finishes execution, and `latest_version` in the cache is strictly greater than `CARGO_PKG_VERSION`, and the session is not suppressed,
- **THEN** a formatted update banner MUST be printed to `stderr`.
- **WHEN** outputting the banner,
- **THEN** `stdout` MUST NEVER receive any banner characters, preserving pipelined command parsing.

### REQ-4: Suppression & Opt-Out Hierarchy
- **WHEN** `--quiet` or `-q` is provided,
- **THEN** update checks and notifications MUST be suppressed.
- **WHEN** environment variable `CE_NO_UPDATE_NOTIFIER=1` is set,
- **THEN** update checks and notifications MUST be suppressed.
- **WHEN** environment variable `CI=true` or `GITHUB_ACTIONS=true` is set,
- **THEN** update checks and notifications MUST be suppressed.
- **WHEN** `stderr` is not an interactive terminal (`!std::io::stderr().is_terminal()`),
- **THEN** update notifications MUST be suppressed.
- **WHEN** `state.json` contains `"update_notifier": { "enabled": false }`,
- **THEN** update checks and notifications MUST be suppressed.

### REQ-5: Tooling & Doctor Integration
- **WHEN** `ce-ai doctor` is executed,
- **THEN** it MUST report the status of the update notifier (`doctor-info: update-notifier: ...`).
- **WHEN** `ce-ai self-update --check` is executed,
- **THEN** it MUST synchronously refresh `<config_dir>/cache/update_check.json`.
