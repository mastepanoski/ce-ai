# Design: Post-Merge OpenSpec Archival & Turn-0 Prescriptions

## System Architecture

```
                    ┌───────────────────────────┐
                    │       AI Agent / Turn 0   │
                    └─────────────┬─────────────┘
                                  │
                                  ▼
                    ┌───────────────────────────┐
                    │    ce-ai workflow resume  │
                    └─────────────┬─────────────┘
                                  │
                                  ▼
                    ┌───────────────────────────┐
                    │ Scan openspec/changes/    │
                    │ for completed tasks.md    │
                    └─────────────┬─────────────┘
                                  │
                 ┌────────────────┴────────────────┐
                 │                                 │
                 ▼ (completed packages found)      ▼ (clean)
  ┌───────────────────────────────┐  ┌───────────────────────────────┐
  │ Emit High-Visibility Action:  │  │ Output Standard               │
  │ "action required: run         │  │ Re-hydrated Context           │
  │  'ce-ai archive <feature>'"   │  │                               │
  └───────────────────────────────┘  └───────────────────────────────┘
```

## Changes to Surfaces

### 1. `AGENTS.md` & `src/commands/init_prj.rs` (Invariant #10)
Update Invariant #10 to:
```markdown
10. **Post-Merge Lifecycle & Clean State**: Immediately after merging a PR, switch to `main`, run `git pull`, prune remotes (`git fetch --prune`), and run `ce-ai workflow status`. If any completed OpenSpec changes exist, the AI agent MUST run `ce-ai archive <feature>` (and submit the corresponding archive PR) so the repository FSM is left at `✓ Ready (100%)` with zero unarchived change warnings before concluding.
```

### 2. `ce-ai workflow resume` in `src/commands/workflow.rs`
In `handle_resume`:
Inspect `openspec/changes/` using `scan_completed_unarchived_changes`.
If completed change packages are detected, print:
```text
! Action Required: OpenSpec change '{change}' is complete ({completed}/{total} tasks). Run 'ce-ai archive {change}' to seal the change package.
```

### 3. `ce-ai doctor` in `src/commands/doctor.rs`
Update finding string:
```text
openspec change '{change_name}' is complete ({completed}/{total} tasks) but not archived — run 'ce-ai archive {change_name}'
```

### 4. `ce-ai workflow status` in `src/commands/workflow.rs`
Update status warning string:
```text
! Warning: 1 OpenSpec change(s) complete but not archived — run 'ce-ai archive <feature>'
```
