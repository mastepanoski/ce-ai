# Exploration: Engram Memory Context Echo & LLM Behavioral Feedback Loop

## Investigation Findings

### 1. `ce-ai` Architecture Verification
In `ce-ai`, the workflow state and stage inference engine reside in `src/commands/workflow.rs`:
- `infer_stage_from_repo` (line 725): Derives stage purely from local Git state (`git status`, `git diff`) and parsing checkboxes in `openspec/changes/<feature>/tasks.md`.
- `maybe_auto_checkpoint` (line 862): Evaluates repo cleanliness and monotonic stage advancement.
- `resume_lines` (line 235): Formats the recovery banner from `state.json` checkpoints.

Neither these functions nor any other logic in `src/commands/workflow.rs` read from Engram or external memory stores. The only appearance of the string `"engram"` in `src/commands/workflow.rs` is a descriptive doc-comment on the `resume` subcommand (line 48). Hypothesis 2 (functional code interaction between Engram and `ce-ai`) is completely refuted.

### 2. Third-Party Engram Plugin Audit
Audit of `github.com/Gentleman-Programming/engram` (Go implementation):
- **Invocation Path**: Context is not injected automatically by session start hooks (`mem_session_start` / `handleSessionStart`, `mcp.go:1934` only records session metadata). Context is only injected when the agent explicitly executes `mem_context` -> `handleContext` (`mcp.go:1613`) -> `Store.FormatContext(project, scope)` (`store.go:3298`).
- **Prompt Recency Window**: `FormatContext` includes "Recent User Prompts" via `RecentPrompts(project, limit 10)` (`store.go:3341-3343`), ordered by `datetime(created_at) DESC LIMIT 10`. It is a hard numerical cutoff of 10 items without semantic relevance filtering or gradual decay.
- **Verbatim Re-Hydration**: `handleSavePrompt` (`mcp.go:1564`) stores prompt text without sanitization. `FormatContext` renders the prompts verbatim (truncated to 200 characters) prefixed with a timestamp. When a prompt previously contained a pasted `ce-ai` status banner, that text reappears in the prompt window for the next 10 turns.

### 3. Cognitive Feedback Loop Mechanics
The loop occurs entirely within the LLM's attention mechanism:
- The LLM reads the re-hydrated context block.
- It encounters past prompt text quoting `tasks progress: 0/N completed`.
- Without explicit framing indicating that past user prompts are historical artifacts, the LLM may attend to the quoted banner as an active operational constraint or current state, replicating the warning in its output or refusing to advance workflow stages.

## Evaluated Documentation Strategies

1. **Keep documentation only in issue tracker**:
   - *Downside*: Knowledge is lost across future agent sessions when the issue is closed.
2. **Document formally in `docs/solutions/architecture/` and `CONCEPTS.md`**:
   - *Benefit*: Permanent knowledge capture compliant with Compound Engineering Stage 6. Helps future agents and human developers distinguish between code bugs and prompt-echo artifacts.
