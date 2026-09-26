---
title: "Automated Background Update Notifier for ce-ai CLI & Harness Releases"
category: "developer-tooling"
date: "2026-09-26"
tags:
  - update-notifier
  - cli
  - release-monitoring
  - fail-closed
  - stderr-banner
  - non-blocking
components:
  - source::update_notifier
  - state::state
  - commands::doctor
  - commands::self_update
  - main
applies_when: "Executing ce-ai commands interactively, running in CI/headless environments, or checking version freshness"
problem_type: "best-practice"
---

# Automated Background Update Notifier for ce-ai CLI & Harness Releases

## Problem
In fast-evolving multi-agent engineering workflows, `ce-ai` frequently releases critical improvements, including prompt injection defenses, updated hard invariants (such as self-explaining PR requirements and glossary accretion guards), and harness compatibility fixes for OpenCode, Claude, and Kimi.

Previously, developers and agents had no proactive notification when a new version was available:
- Outdated binaries remained undetected until manual checks (`ce-ai self-update --check`, `ce-ai status`).
- Naive synchronous network checks would introduce noticeable command latency (hundreds of milliseconds) on every CLI invocation.
- Writing update alerts to standard output would corrupt UNIX pipelines (e.g. `ce-ai workflow status | jq .`).

## Solution
Implemented an asynchronous, fail-closed, throttled Update Notifier architecture adhering to strict non-blocking and pipeline integrity principles:

1. **Throttled Cache Persistence**:
   - Stores metadata (`last_checked_at`, `latest_version`, `latest_tag`, `release_url`) at `<config_dir>/cache/update_check.json`.
   - Evaluates staleness against a configurable TTL (default 24 hours, via `state.json` `"update_notifier": { "enabled": true, "interval_hours": 24 }`).
   - Uses `crate::state::write_atomic` to prevent file corruption from concurrent process writes or sudden exits.

2. **Detached Worker Thread**:
   - When cache is stale or absent, spawns a detached background thread (`std::thread::spawn`) to query the GitHub Releases API with a 3-second timeout.
   - Main command dispatch proceeds immediately on the primary thread with zero latency overhead.
   - Fails completely silently upon network timeouts, rate limits, or air-gapped environments.

3. **Stderr Unicode Box Banner**:
   - Renders a clean Unicode box banner exclusively to `stderr` upon process completion, guaranteeing `stdout` remains pristine for script pipelines.
   - Automatically suppressed when `stderr` is not an interactive terminal (`!std::io::stderr().is_terminal()`), under CI (`CI=true`, `GITHUB_ACTIONS=true`), with `--quiet`, or via `CE_NO_UPDATE_NOTIFIER=1`.

4. **Diagnostics & Manual Tooling**:
   - `ce-ai doctor` surfaces notifier status and cache freshness.
   - `ce-ai self-update --check` synchronously refreshes the cache.

## Verification
- Unit tests in `src/source/tests/update_notifier.rs` verify cache roundtrips, RFC3339 age calculations, semver comparisons, box formatting, and multi-variable suppression logic.
- Integration tests in `tests/cli.rs` verify that banner appears on `stderr` when simulated in interactive terminals, never appears on `stdout`, and respects all opt-out variables.
- Verified zero clippy warnings and 100% clean formatting.
