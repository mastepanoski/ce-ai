# Design: TUI Workflow Panel — Native Action Execution

## Component Boundaries

```
src/
├── commands/workflow.rs   # per-action fns return Result<Vec<String>, CeError>; run() prints
└── tui.rs                 # run_workflow_cmd consumes lines; failure-class helper; guide rework
docs/user-guide/workflow-panel.md  # explanation doc (Diátaxis: explanation intent)
```

## Data Flow

```
[Enter] ─▶ execute_action("Workflow FSM Status")
             └▶ status_lines(ctx) -> Result<Vec<String>, CeError>   (workflow.rs)
                  ├─ Ok(lines)  ─▶ modal success block
                  └─ Err(e)     ─▶ modal failure block (❌ + actionable copy)
[1-7]    ─▶ checkpoint_lines(ctx, phase, task) -> Result<Vec<String>, CeError>
```

Directional sketch only — not implementation specification.

## Key Contracts

- `run(ctx, args)` signature and CLI output format unchanged; exit codes still map through `CeError` (0=success, 3=state, 4=io).
- Status content derives from real state reads: current phase/task from checkpoint entry in `State.last_update_check`; absence renders the existing "(No progress checkpoint saved yet…)" hint as a line, not canned filler.
- Failure copy class: single helper maps any `CeError` to a stable two-line block (`❌ <short cause>` + remedy hint). Shared by status and checkpoint actions.
- Guide markers: `[run]` prefix for native actions, `skill:` prefix for agent-session stages — text-perceivable, no color dependence. Verify row copy references "the project's test/e2e commands".
- Footer hint enumerates `[Enter]` and `[1-7]` only.

## Docs Section Shape

`docs/user-guide/workflow-panel.md`, explanation intent: what each action does natively → why agent stages are guide-only (harness-specific delegation across 12 harnesses; single-dashboard experience) → stage-to-skill mapping table (opencode naming) → resume-exclusion note. Must satisfy AE5 checklist. README gets at most a one-line map entry if the ≤100-line budget holds.

## State / Compatibility

No schema changes. No new state fields. All reads go through `State::load`; any future mutation stays behind `write_atomic`.
