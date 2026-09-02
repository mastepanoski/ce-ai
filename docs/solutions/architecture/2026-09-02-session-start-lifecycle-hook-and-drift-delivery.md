---
module: harness
tags: [hooks, session-start, claude-code, repo-state, workflow-resume, adoption-block, v4]
problem_type: architecture
---

# Guaranteed Turn-0 Drift Delivery via Native Lifecycle Hooks & Enforced Directives

## Problem
In `ce-ai v1.30.0`, `workflow resume` implemented live sub-15ms `RepoState` drift detection, but relied entirely on the AI agent spontaneously choosing to invoke `ce-ai workflow resume`. In runtimes without native hooks, agents frequently forgot to call it, leaving the Turn-0 environment un-synchronized and suffering from the 5–8 turns of observation lag documented in arXiv:2608.26263v2.

## Solution
Implemented a 4-tier defense-in-depth architecture:

1. **Native Claude Code Lifecycle Hook (`src/harness/claude.rs`):**
   - Implemented `ensure_session_start_hook` and `remove_session_start_hook`.
   - `ce-ai init-prj` non-destructively injects a `SessionStart` command hook into `.claude/settings.json`:
     ```json
     {
       "hooks": {
         "SessionStart": [
           {
             "matcher": ".*",
             "hooks": [
               {
                 "type": "command",
                 "command": "ce-ai workflow resume"
               }
             ]
           }
         ]
       }
     }
     ```
   - Automatically executed by Claude Code at session start, streaming `RepoState` into system context before the first user turn.

2. **Universal Turn-0 Directive & Block Version Bump (`BLOCK_VERSION: 4`):**
   - Added explicit mandatory Turn-0 prompt directives to `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs`.
   - Bumped `BLOCK_VERSION` 3 → 4, enabling `ce-ai doctor` and `ce-ai status` to flag stale v3 blocks and prompt re-adoption.

3. **Runtime Checkpoint & Doctor Verification Gates (`src/commands/workflow.rs`, `src/commands/doctor.rs`):**
   - `ce-ai workflow checkpoint` runs `probe_repo_state()` and surfaces non-blocking drift warnings if managed files drifted during the turn.
   - `ce-ai doctor` checks adopted projects for `.claude/settings.json` hook health and reports `project-adoption: Claude Code SessionStart hook missing` if omitted.

4. **Accurate Documentation Alignment:**
   - Updated `docs/user-guide/zero-step-drift-recovery-explained.md` and `docs/user-guide/harnesses-loops-and-context-masterclass.md` to clearly define hook vs directive delivery.
   - Removed obsolete placeholder claim from `docs/user-guide/workflow-panel-native-vs-agent-skills.md`.

## Key Learnings
1. **Self-Healing Re-Adoption:** In `init_prj.rs`, when `existing_block == full_block`, skipping the rewrite of `AGENTS.md` must not short-circuit the repair of derived harness files (such as `.claude/settings.json` hooks or `.cursorrules`). Running derived updates on every `init-prj` invocation makes the CLI self-healing without altering `AGENTS.md` mtime.
2. **Preserving User Hook Arrays:** Hook configurations in `.claude/settings.json` may contain arbitrary user matchers and commands. Merging must target specifically `hooks.SessionStart` with `matcher: ".*"`, appending only our command without re-writing or clearing user hooks.
