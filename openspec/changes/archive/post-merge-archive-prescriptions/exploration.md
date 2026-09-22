# Exploration: Post-Merge OpenSpec Archival & Turn-0 Prescriptions

## Investigation
AI agents operating on `ce-ai` projects are governed by two primary surfaces:
1. **Agent Instruction Blocks**: Injected into `AGENTS.md`, `CLAUDE.md`, and `GEMINI.md` by `ce-ai init-prj`.
2. **Turn-0 CLI Directives**: The command `ce-ai workflow resume` executed at every session start or context reset.

### 1. Invariant #10 in `AGENTS.md`
Currently, Invariant #10 states:
```markdown
10. **Post-Merge Cleanup**: Immediately after merging a PR, switch to `main`, run `git pull`, delete merged local branches (`git branch -d`), prune remotes (`git fetch --prune`), and remove turn-created temporary worktrees.
```
Notice that it focuses strictly on git housekeeping. Once an AI agent cleans up local branches and worktrees, it evaluates that its contract is satisfied and stops calling tools.

### 2. Turn-0 Directives in `ce-ai workflow resume`
In `src/commands/workflow.rs`, `handle_resume` rehydrates:
- Current branch
- Working tree cleanliness
- Manifest SHA256 integrity
- Adoption block status
- Readiness advisory

However, it does not scan `openspec/changes/` for packages whose `tasks.md` has `all_tasks_complete() == true`. When an agent switches to `main` and runs `ce-ai workflow resume`, it receives `✓ Ready (100%)` and proceeds to look for a new task, leaving the previous change package unarchived indefinitely.

### 3. Prescriptive Diagnostic Messaging
In `src/commands/doctor.rs`:
```rust
findings.push(format!(
    "openspec change '{change_name}' is complete ({completed}/{total} tasks) but not archived — see openspec/changes/archive/README.md"
));
```
Referring an automated LLM agent to a README markdown file introduces ambiguity. Directly outputting the actionable command (`run 'ce-ai archive {change_name}'`) enables deterministic tool invocation.

## Evaluated Options

- **Option A: Auto-archiving during `ce-ai workflow resume` silently on `main`**:
  *Trade-off*: Could violate branch-protection invariants if `main` requires PRs. Committing directly to `main` without a PR breaks Hard-Gate Invariant #1.
- **Option B: Prescriptive Turn-0 Guidance + Governance Contract (Chosen)**:
  *Rationale*: Preserves deterministic security and PR invariants. Emits a high-visibility, prescriptive instruction:
  `action required: OpenSpec change '<feat>' is complete. Run 'ce-ai archive <feat>' to seal the change package.`
  Updates Invariant #10 and adoption block templates to mandate this behavior for all AI harnesses.
