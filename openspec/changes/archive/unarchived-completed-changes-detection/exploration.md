# Exploration: Architectural Levers for OpenSpec Ledger Completeness Detection

## 1. Investigation of Existing Probes
The existing tasks desync probe in `src/commands/workflow.rs` (`probe_openspec_context_in`) resolves an active feature by inspecting:
1. The currently active workflow feature stored in `state.json`.
2. The current git branch name sanitized against feature directory names.
3. Fallback: the most recently modified directory in `openspec/changes/`.

While this works well for active work, it completely ignores all other directories in `openspec/changes/`. If a feature was merged and its branch deleted, or if work transitioned to another feature, completed folders remain dormant in `openspec/changes/`.

## 2. Invocation Surfaces: Discretionary vs Non-Discretionary
A naive approach would only implement this check as a health probe in `ce-ai doctor`. However, this suffers from two structural flaws:
1. **Developer / Agent Workflow Discretion:** In non-compound development loops (e.g. ad-hoc bugfixes or hotfixes), developers and agents do not run `ce-ai doctor`. The blind spot persists.
2. **Turn-0 Invariant:** `ce-ai`'s architectural guarantee across all supported harnesses (Claude Code, OpenAI Codex, GitHub Copilot, Pi, OpenCode, Cursor, Antigravity) is grounded in Turn-0 state delivery via `ce-ai workflow resume` (injected via `SessionStart` / `PreInvocation` hooks).

By embedding the ledger summary in `RepoState` and rendering a compact summary line in `resume_lines`, the warning is deterministically surfaced at the very start of every session without requiring manual CLI invocation.

## 3. Evaluated Tradeoffs

### Tradeoff 1: Automated Archiving (`git mv`) vs Diagnostic Notification
- **Option A (Automated git mv):** When `probe_repo_state` detects a completed folder, automatically execute `git mv` to `openspec/changes/archive/`.
  - *Cons:* Violates the principle that `ce-ai` does not perform unrequested git staging or tree mutations in developer repositories. Could race with active commits or worktrees.
- **Option B (Diagnostic Notification - Selected):** Surface the drift in Turn-0 `resume`, `status`, `checkpoint`, and `doctor`, leaving actual archival to the developer or explicit agent chore.
  - *Pros:* Safe, non-mutating, zero unexpected working tree dirtiness.

### Tradeoff 2: Data Representation in `RepoState`
- **Option A (`Vec<String>` of feature names):**
  - *Cons:* Requires `ce-ai doctor` to re-read disk and re-parse `tasks.md` to format the `(N/N tasks)` string.
- **Option B (`Vec<UnarchivedChange>` containing `feature`, `completed_tasks`, `total_tasks` - Selected):**
  - *Pros:* Fully self-contained, typed, serializable into JSON for `--json` consumers, and eliminates duplicate I/O across commands.
