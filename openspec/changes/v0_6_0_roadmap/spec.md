# OpenSpec Specification: Release v0.6.0 Roadmap

## Behavioral Scenarios

### Scenario 1: TUI Workflow Dashboard
WHEN the user launches `ce-ai tui` and navigates to the `Workflow` tab
THEN it MUST render the current 7-Stage FSM state, active task string, and checkpoint history from `state.json`.

### Scenario 2: Extended Health Diagnostics
WHEN the user runs `ce-ai doctor`
THEN it MUST include companion tool health status for Engram, CodeGraph, Context7, and RTK in its diagnostic findings.

### Scenario 3: Real-Time Sync Watcher
WHEN the user runs `ce-ai sync --watch`
THEN it MUST continuously monitor managed configuration paths and automatically re-sync drift upon detecting file mutations.
