# Brainstorm & Requirements: Session-Start Drift Delivery & Hook Architecture

**Date:** 2026-09-02  
**Status:** Approved  
**Topic:** Guaranteed Turn-0 Delivery of `RepoState` & `workflow resume` across AI Harnesses

---

## 1. Problem Statement

In `v1.30.0`, `ce-ai` introduced Zero-Step Environment Drift Recovery via `ce-ai workflow resume`. The live probing engine calculates `RepoState` (Git working tree, branch, HEAD SHA, plugin manifest SHA256 integrity, and adoption status) in <15ms.

However, an operational gap exists:
- `ce-ai workflow resume` is a pure CLI subcommand.
- No harness lifecycle hooks (`SessionStart`, `PostToolUse`, etc.) were installed by `ce-ai`.
- The managed block injected in `AGENTS.md` (and copied to `CLAUDE.md`, `.cursorrules`, etc.) did not mandate running `ce-ai workflow resume`.
- The system relied 100% on the AI agent "remembering" to invoke `workflow resume` on its own.
- Documentation claimed "Autonomous harnesses run `ce-ai workflow resume --json`" and described the full lifecycle as "zero-step", overpromising automated delivery.

---

## 2. Solution: 4-Tier Defense-in-Depth Delivery Architecture

To reliably bridge this gap without breaking harnesses that do not support shell hooks:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        Session Startup / Resume                        │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
           ┌────────────────────────┴────────────────────────┐
           ▼                                                 ▼
[Layer 1: Native Platform Hook]              [Layer 2: Universal Textual Directive]
 Claude Code: .claude/settings.json           All Harnesses: AGENTS.md (v4 block)
 SessionStart hook runs workflow resume       Mandatory Turn-0 resume directive
           │                                                 │
           └────────────────────────┬────────────────────────┘
                                    ▼
                 [Layer 3: Runtime Verification Gate]
                  • workflow checkpoint checks RepoState
                  • doctor checks Claude hook configuration
                                    │
                                    ▼
                 [Layer 4: Truthful Documentation]
                  • Zero-Step applies to state injection
                  • Delivery mechanisms accurately documented
```

### Layer 1: Native Shell Lifecycle Hook (`SessionStart`)
- Target: Claude Code (which natively supports project-level `.claude/settings.json` hooks).
- Mechanism:
  - When `ce-ai init-prj` runs on a project with `.claude/` or when initializing Claude support, it non-destructively merges a `SessionStart` hook in `.claude/settings.json` using atomic writes (`write_atomic`).
  - Hook configuration:
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
  - `ce-ai deinit-prj` cleans up the injected hook from `.claude/settings.json`, removing the file only if it was created by `ce-ai` and contains no other user settings.

### Layer 2: Universal Textual Directive & Block Version Bump (`BLOCK_VERSION: 4`)
- Target: All harnesses (OpenCode, Cursor, Copilot, Codex, Pi, Agy, Grok, Kimi, Fx, Custom).
- Mechanism:
  - Add an explicit **Turn-0 Directive** to `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs`:
    ```markdown
    ### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)
    At the start of EVERY new turn, session, or after context compaction, before running any task or reading historical chat assumptions, the AI agent MUST run:
    `ce-ai workflow resume`
    to synchronize live Git working tree state, active branch, manifest SHA256 integrity, and active OpenSpec progress.
    ```
  - Bump `BLOCK_VERSION: u32 = 4` in `src/commands/init_prj.rs`.
  - Existing adopted projects at `v=3` will be flagged as `StaleVersion` by `ce-ai doctor` and `ce-ai status`, providing automated upgrade instructions (`re-run ce-ai init-prj --tier <tier> to upgrade`).

### Layer 3: Runtime Verification Gate
- `ce-ai workflow checkpoint <stage> <task>`:
  - In addition to FSM transitions, `checkpoint` runs `probe_repo_state()`.
  - Emits the current environment state and drift warning if `manifest_drift_count > 0` or dirty files are present.
- `ce-ai doctor`:
  - When inspecting an adopted project that has a `.claude/` directory, checks if `.claude/settings.json` contains the `SessionStart` hook for `ce-ai workflow resume`.
  - If missing, reports a targeted finding: `claude-hook-missing: Claude Code SessionStart hook missing at '<path>/.claude/settings.json' — re-run ce-ai init-prj to configure`.

### Layer 4: Accurate Documentation
- `docs/user-guide/zero-step-drift-recovery-explained.md`:
  - Clarify that "Zero-Step" describes the zero-observation-lag state injection upon resume (sub-15ms probe, deterministic SHA256), while session startup relies on the hook (automated) or the Turn-0 directive (textual).
- `docs/user-guide/harnesses-loops-and-context-masterclass.md`:
  - Update line 111 to explain the dual-delivery model (hook vs textual directive).
- `docs/user-guide/workflow-panel-native-vs-agent-skills.md`:
  - Replace the outdated placeholder text in line 44 with the real `v1.30.0`+ live probing engine capabilities.

---

## 3. Invariants & Boundaries

1. **Non-Destructive Hook Merging:** NEVER overwrite user hooks in `.claude/settings.json`. Always parse existing JSON, append/update only the `ce-ai workflow resume` entry, and write atomically.
2. **Deterministic Block Bump:** Bumping `BLOCK_VERSION` 3 $\to$ 4 must be accompanied by updating all test fixtures in `tests/cli.rs` and docs.
3. **Graceful Fallback:** If `.claude/settings.json` cannot be parsed or if the user is in a non-Claude harness, the textual directive in `AGENTS.md` remains 100% intact.
