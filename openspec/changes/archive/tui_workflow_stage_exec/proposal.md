# Proposal: TUI Workflow Panel — Native Action Execution

## Problem

The Workflow (FSM) panel in the `ce-ai` TUI dashboard (issue #76) advertises the 7-stage cycle and its mapped commands but is read-only: every real action requires quitting to a terminal. Most advertised commands are agent-harness skills the binary cannot execute, and some suggestions are project-specific (`cargo test`), making hardcoded execution wrong for most users.

## Proposal

Make the panel honestly actionable for what ce-ai natively supports:

1. `[Enter]` renders the **real** `workflow status` output in the result modal (today: canned success text).
2. `[1-7]` stage checkpoints keep working unchanged.
3. Stage rows distinguish native actions from agent-session skills with non-color markers (`[run]` vs `skill:`), naming each stage's opencode skill.
4. Command failures render a defined failure modal class instead of silent/canned output.
5. A teacher-style docs section explains why agent stages are guide-only.
6. The `workflow resume` keybinding is deliberately excluded this iteration (print-only stub; adds nothing over `[1-7]`).

## In Scope

- Refactor of workflow command functions to return output lines (CLI behavior preserved).
- TUI render changes for the Workflow tab only.
- One explanation doc under `docs/user-guide/`.
- SemVer/CHANGELOG governance.

## Out of Scope

- Executing project-specific commands from the TUI; harness delegation or external terminals; clipboard features; Dry-Run in this panel; async TUI rework; resume keybinding.

## Risks

- CLI output formatting drift during refactor → mitigated by keeping `println!` at the `run()` boundary plus an integration test diffing CLI output.
- Marker prefixes crowding narrow terminals → short markers only.

## Success Criteria

- Full FSM query/checkpoint lifecycle driven from the panel without terminal round-trips or memorized flags.
- The panel never suggests running something it cannot run.
- Docs pass the AE5 classification checklist; planning scope fully resolved.

Origin requirements: docs/brainstorms/tui-workflow-stage-exec-requirements.md
