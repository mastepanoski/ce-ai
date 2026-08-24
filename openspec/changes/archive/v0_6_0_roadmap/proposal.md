# OpenSpec Proposal: Release v0.6.0 Roadmap

## Problem Statement
While `ce-ai` v0.5.0 introduced workspace scope isolation (`--scope workspace`), companion tools management (`ce-ai tools`), and progress checkpointing (`ce-ai workflow`), users currently need manual invocation to inspect FSM states and check companion tool health. Release v0.6.0 expands `ce-ai` into a proactive workflow environment by adding:
1. **Interactive FSM Dashboard in TUI**: Real-time visualization of the 7-stage Compound Engineering Flywheel in `ce-ai tui`.
2. **Enhanced Companion Health Diagnostics**: In-depth database integrity checks for Engram (SQLite), CodeGraph watcher status, and RTK binary availability in `ce-ai doctor`.
3. **Background Auto-Sync & Drift Watcher**: Optional daemon mode (`ce-ai sync --watch`) to maintain SHA256 integrity across harnesses in real-time.

## Out of Scope
- Direct binary compilation of sidecars (Engram/CodeGraph binaries are managed upstream).
- Cloud telemetry or external data collection.

## Success Criteria
- `ce-ai tui` renders a dedicated **Workflow (FSM)** tab displaying active stage, checkpoint history, and companion status.
- `ce-ai doctor --extended` runs SQLite integrity checks on Engram DB and probes CodeGraph index staleness.
- All changes maintain 100% backward compatibility and 0 compiler warnings (`-D warnings`).
