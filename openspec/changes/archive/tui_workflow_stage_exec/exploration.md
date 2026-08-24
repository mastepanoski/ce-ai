# Exploration: TUI Workflow Panel — Native Action Execution

## Options evaluated for surfacing command output in the TUI modal

### Option A — Capture stdout of `workflow::run` (rejected)

The TUI owns stdout via crossterm raw mode and ratatui's alternate screen; any `println!` emitted mid-loop fights the renderer. In-process invocation is how every existing tab action works. Dead end.

### Option B — Refactor workflow commands to return output lines (chosen)

Internal per-action functions return `Result<Vec<String>, CeError>`; the public `run(ctx, args)` prints them via `println!`, preserving CLI behavior and the strict exit-code mapping. This mirrors the dominant dashboard pattern (`run_*_cmd -> Vec<String>` at src/tui.rs:828-1052) and gives the TUI a consumable value.

### Option C — Spawn agent harnesses non-interactively for stage commands (`opencode run`, etc.) — rejected

Technically possible but rejected as product choice: delegation paths are harness-specific across 12 supported harnesses, and spawning external processes breaks the single-dashboard experience and output capture. Recorded as the taught rationale, not framed as impossibility.

## Resume keybinding investigation

Codebase verification (round-1 review): `Action::Resume` never inspects checkpoint state — it prints "resuming…" and delegates to `status()`. Checkpoints live in `State.last_update_check` (`{phase} | {task} | {timestamp}`), with no dedicated field; `State::load` returns defaults when `state.json` is missing (no panic path). Since `[1-7]` already deliver reversible stage transitions, a resume binding would show false success — violating honest execution. Dropped this iteration; real recovery is a candidate follow-up requiring a defined user delta (e.g., restoring non-stage state).

## Dry-Run evaluation

Excluded: status never mutates `state.json`, no resume key ships, checkpoint transitions are trivially reversible via re-pressing the prior stage key. Verified against code, not assumed.

## Prior art

- docs/solutions/architecture/proactive-workflow-observability-fsm-tui-sync-watcher.md — established Workflow-tab render pattern.
- docs/solutions/backup-restore-management-and-point-in-time-recovery.md — validate-then-propagate error conventions.
- openspec/changes/model_assignments_resilience_and_tui/ — structural template for this change folder.
