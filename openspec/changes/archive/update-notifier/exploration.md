# Exploration: Automated Background Update Notifier

## Technical Investigation

### Existing Infrastructure
1. **GitHub Release Resolution (`src/source/binary_release.rs`)**:
   - `resolve_latest_cli_release(&client, token.as_deref())` queries `https://api.github.com/repos/mastepanoski/ce-ai/releases/latest`.
   - Has fallback to redirect header extraction from `https://github.com/mastepanoski/ce-ai/releases/latest` and atom feed `https://github.com/mastepanoski/ce-ai/releases.atom`.
   - `compare_cli_versions(a, b)` performs semver-aware component comparisons (`v1.10.0` > `v1.9.0`).
2. **Atomic Writes (`src/state/mod.rs`)**:
   - `write_atomic(path, bytes)` guarantees crash-consistent writes using tempfile + rename.
3. **Configuration & State (`src/state/state.rs`)**:
   - `State` struct in `~/.ce-ai/state.json`. Already contains `last_update_check: Option<String>` and `latest_release_tag: Option<String>`, but can be complemented by a dedicated cache file at `<config_dir>/cache/update_check.json` and config entry `UpdateNotifierConfig`.
4. **Context & Execution Flow (`src/main.rs`, `src/commands/mod.rs`)**:
   - `main()` parses CLI args, initializes `Context`, and calls `ce_ai::commands::registry::dispatch(&ctx, cli.command)`.
   - Any banner printed at exit can be executed right before `std::process::exit`.

### Evaluated Options

#### Option A: Synchronous Check with Timeout in `main.rs`
- Description: Before or after command execution, make a blocking HTTP GET request with a short timeout (e.g. 500ms).
- Pros: Simple single-threaded code.
- Cons: Introduces up to 500ms of latency to every CLI command, noticeable to humans and agents; fails user expectation of instant CLI response.
- Decision: **Rejected**. Violates non-blocking performance requirement.

#### Option B: Detached Background Daemon / Cron Job
- Description: Install an OS background service (systemd or launchd) or cron job to check for updates periodically.
- Pros: Decoupled from CLI process lifecycle.
- Cons: Extreme administrative complexity, requires system permissions, platform-specific daemons (macOS launchd vs Linux systemd vs Windows Task Scheduler), and creates unwanted background resource usage.
- Decision: **Rejected**. Overkill for a developer CLI tool.

#### Option C: Throttled Cache Read + Non-Blocking Background Worker Thread (Selected)
- Description:
  1. On CLI launch, read local cache `<config_dir>/cache/update_check.json` (sub-millisecond disk read).
  2. If cache is missing or older than 24 hours (and not suppressed), spawn a background worker thread (`std::thread::spawn`) to query the GitHub Releases API with a short timeout (3s) and write the updated cache atomically.
  3. Meanwhile, the main thread immediately proceeds to execute the CLI subcommand with 0ms added delay.
  4. When the subcommand finishes: if the cached (or just-fetched) latest version is newer than current, and `stderr` is an interactive TTY, render the notification banner to `stderr`.
- Pros:
  - 0ms latency impact on CLI command dispatch.
  - Fail-closed: offline or rate-limited runs do not emit errors or slow down the CLI.
  - `stdout` remains pristine for piping (`jq`, file redirection).
  - No background daemons or OS-specific service registrations.
- Decision: **Accepted**.

## Trade-offs & Mitigations
- **Thread Lifetime on Fast Exit**: If a subcommand finishes in 5ms (e.g. `--help`), a newly spawned background worker thread might not finish fetching before `std::process::exit()` terminates the process.
  *Mitigation*: This is standard for update notifiers (e.g. `npm`, `gh`, `brew`). The thread will complete on slightly longer commands, or the user can explicitly trigger `ce-ai self-update --check` to refresh immediately.
- **Terminal Detection Across Platforms**: `std::io::stderr().is_terminal()` accurately detects whether `stderr` is an interactive terminal across Unix and Windows.
- **Suppression Precedence**:
  1. Global flag `--quiet` / `-q`
  2. Env `CE_NO_UPDATE_NOTIFIER=1`
  3. Env `CI=true` or `GITHUB_ACTIONS=true`
  4. `!std::io::stderr().is_terminal()`
  5. Config `update_notifier.enabled == false`
  If any matches, the update notifier is completely disabled (no network, no threads, no banners).
