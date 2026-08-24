# Proposal: Real Long-Running `sync --watch` Loop & Drift Recovery

## Problem Statement
Issue #159 notes that `ce-ai sync --watch` currently prints a monitoring message and exits immediately after a single sync pass, failing to detect or repair subsequent drift.

## Proposed Solution
Implement a real long-running watcher loop in `src/commands/sync.rs`:
1. Signal handling for Ctrl-C / SIGINT (`RUNNING` atomic boolean flag).
2. Debounced polling loop checking managed file hashes against `install-manifest.json` and desired source tree.
3. Drift detection & automatic repair: when drift is detected, `sync_with` automatically repairs files and updates `install-manifest.json` + `state.json`.
4. Error resilience: failures in a single sync pass (e.g. temporary file lock) log a notice and keep watching without crashing silently.
5. Clean termination: on Ctrl-C / SIGINT, print summary of checks performed and drift repairs executed, exiting with code 0.
6. Testable interface: support `--max-passes` and `--interval-ms` flags for deterministic, non-blocking unit and integration tests.

## Acceptance Criteria
- `sync --watch` continuously monitors managed paths until SIGINT or `--max-passes` limit is reached.
- Mid-watch file mutations (drift) are automatically detected and repaired within the polling window.
- Terminal exit on Ctrl-C / SIGINT is clean (exit code 0).
- Sync pass errors are logged to stderr without crashing the watch loop.
