# Specification: Real Long-Running `sync --watch` Loop & Drift Recovery

## Requirements

### R1: Long-Running Monitoring Loop & Signal Handling
WHEN `ce-ai sync --watch` is executed,
THEN `ce-ai` SHALL continuously monitor managed files at the specified interval until SIGINT or `--max-passes` limit is reached, safely handling single or multiple signal handler registrations in test environments.

### R2: Performance-Aware Drift Detection & Automatic Repair
WHEN `sync --watch` runs a polling tick,
THEN `ce-ai` SHALL perform an in-memory hash comparison (`diff::diff`) first and execute disk sync repair (`sync_with`) ONLY when drift actions are present.

### R3: Dry-Run Watch Behavior
WHEN `ce-ai --dry-run sync --watch` is executed,
THEN `ce-ai` SHALL log detected drift actions on each tick without modifying files on disk or updating state.

### R4: Failure Resilience & Clean Exit
WHEN a sync pass encounters a temporary error or when SIGINT is received,
THEN `ce-ai` SHALL log the error to stderr without crashing the loop, and exit cleanly (code 0) with a summary upon termination.
