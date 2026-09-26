# Proposal: Automated Background Update Notifier for ce-ai CLI & Harness Releases

## Problem Statement
`ce-ai` orchestrates AI agent governance, workflow state machines, and harness adapters across development environments. Upstream releases frequently ship critical bug fixes, security patches (e.g. prompt injection guards, secret redaction), agent directives, and harness compatibility updates.

Currently, developers and AI coding agents have no proactive notification when a new version of `ce-ai` is released:
- Users must manually run `ce-ai self-update --check`, `ce-ai status`, or visit GitHub releases to discover updates.
- Running outdated versions causes silent behavioral drift between development workstations, CI runners, and agent personas.
- In headless scripts and agent toolchains, any intrusive or blocking network check could break pipelines (`stdout` corruption) or introduce unacceptable command latency.

## Scope Boundaries

### In Scope
1. **Throttled Update Cache**: Check GitHub Releases API for `mastepanoski/ce-ai` at most once every 24 hours (configurable via `update_check_interval_hours: 24`). Cache stored at `<config_dir>/cache/update_check.json`.
2. **Non-Blocking Background Thread**: Background check spawned in a detached thread when cache is stale or missing; zero delay on CLI command dispatch.
3. **Fail-Closed Resilience**: Complete silence and zero latency penalty when offline, in an air-gapped environment, or rate-limited by GitHub API.
4. **Non-Intrusive Terminal Banner on `stderr`**: Formatted notification printed to `stderr` upon process completion when a newer release is detected, leaving `stdout` pipelines (e.g. `ce-ai ... | jq`) pristine.
5. **Multi-Layer Suppression & Opt-Out**: Automatic suppression under CI (`CI=true`, `GITHUB_ACTIONS=true`), non-TTY `stderr`, global `--quiet` flag, environment variable `CE_NO_UPDATE_NOTIFIER=1`, or configuration toggle `update_notifier.enabled: false`.
6. **Diagnostics & Tooling Integration**: Surface notifier status in `ce-ai doctor` and refresh cache in `ce-ai self-update --check`.

### Out of Scope
- Automatic background binary downloading or silent in-place binary patching without explicit user invocation (`ce-ai self-update` handles explicit upgrades).
- Blocking command execution to wait for network responses.
- Modifying homebrew tap formulas directly from the CLI.

## Risk Evaluation & Mitigation
- **Latency / Performance Risk**: If the CLI waits for a network request, command execution feels sluggish.
  *Mitigation*: Network request runs in a background thread; main thread immediately dispatches the subcommand and never blocks on the thread.
- **Pipeline Corruption Risk**: Writing notifications to `stdout` breaks piping into `jq` or file redirects.
  *Mitigation*: Notifications are written strictly to `stderr` and only when `stderr` is an interactive terminal (`is_terminal()`).
- **Rate Limiting Risk**: Unauthenticated GitHub API calls are limited to 60 requests/hour per IP.
  *Mitigation*: 24-hour cache TTL throttling, optional token authentication via `CE_AI_GITHUB_TOKEN`, and fallback to Atom feed / redirect headers if rate-limited.
- **State Corruption Risk**: Concurrent CLI invocations writing to `update_check.json`.
  *Mitigation*: Atomic tempfile-and-rename writes via `crate::state::write_atomic`.

## Success Criteria
- Subcommand execution latency is unaffected (0ms added delay).
- No output printed to `stdout`; `stderr` receives the notification banner only when a newer version is verified and output is a TTY.
- Running in CI or with `CE_NO_UPDATE_NOTIFIER=1` produces zero network calls and zero notifications.
- `ce-ai doctor` displays accurate update notifier health and cache status.
- 100% passing tests including unit tests, integration tests, clippy, and formatting.
