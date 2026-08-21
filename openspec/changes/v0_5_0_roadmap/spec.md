# OpenSpec Specification: Release v0.5.0 Roadmap

### WS-1: Workspace Scope Installation
- **WHEN** `ce-ai install --scope workspace` is executed,
- **THEN** `ce-ai` MUST resolve the git repository root and install managed skills and configs locally inside the repository.

### TM-1: Companion Tools Manager
- **WHEN** `ce-ai tools status` is executed,
- **THEN** `ce-ai` MUST report the installation and MCP configuration status of Engram, CodeGraph, Context7, and RTK.

### WF-1: Workflow FSM & Progress Recovery
- **WHEN** `ce-ai workflow status` or `ce-ai workflow checkpoint` is executed,
- **THEN** `ce-ai` MUST record and display the current 7-stage workflow checkpoint and OpenSpec task progress.
