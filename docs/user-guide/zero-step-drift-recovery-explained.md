# Zero-Step Environment Drift Recovery Explained

> **Audience:** Beginner / Newcomer  
> **Intent (Diátaxis):** Explanation  
> **Prerequisites:** Basic understanding of Git and terminal commands.

---

## 1. The Real-World Problem: The "Stale Whiteboard" Analogy

Imagine you are working with a human teammate in a shared office. 

On Monday morning, you write the project plan on the office whiteboard:
> *"We are currently working on the `main` branch, and the file `auth.rs` has no uncommitted changes."*

Later that afternoon, while your teammate steps out for coffee, you switch branches to `feat/login` and edit three files in the repository.

When your teammate returns, if they **only look at the whiteboard from Monday morning** instead of checking the actual computer screen, they will:
1. Assume they are still on `main`.
2. Assume `auth.rs` is untouched.
3. Write new code based on old assumptions, causing merge conflicts, overwriting your work, or failing builds.

In AI engineering, this problem is called **Environment Drift**.

```
[Human Developer] ─── modifies files / switches branch ───► [Disk Reality]
                                                                  ▲
                                                                  │ (DRIFT!)
[AI Agent Context] ─── remembers old chat history ───────────────┘
                       (thinks nothing changed)
```

---

## 2. Why AI Agents Get Confused (The 5–8 Turn Lag)

Large Language Models (LLMs) used in coding assistants (Claude Code, Cursor, GitHub Copilot, OpenCode, Codex, Antigravity) are **history-accumulating runtimes**. 

When an AI agent wakes up or resumes after a context compaction:
- It reads its **chat history** first.
- If the chat history says *"We are on step 3 of building feature X"*, the AI agent focuses intensely on those previous words.
- Even if files changed on disk while the agent was idle, the agent **does not automatically know it**.

A landmark 2026 research paper by Badhe et al. (*SKILL.state: Scalable Long-Horizon Agent Skills*, [arXiv:2608.26263v2](https://arxiv.org/html/2608.26263v2)) measured this exact phenomenon. They discovered that standard agents take **5 to 8 consecutive turns of trial-and-error** just to realize that the workspace changed behind their backs.

Those 5 to 8 wasted turns burn hundreds of thousands of tokens, increase latency, and frustrate developers.

---

## 3. How `ce-ai` Fixes This: Zero-Step Recovery

`ce-ai` eliminates this lag completely by introducing **Turn-0 Ground-Truth Synchronization** through a live, deterministic probe called `RepoState`.

Instead of asking the AI agent to guess what changed, `ce-ai workflow resume` inspects the disk at the exact millisecond of resumption and injects a crystal-clear snapshot of reality into the prompt.

```
┌────────────────────────────────────────────────────────┐
│               ce-ai workflow resume                    │
└──────────────────────────┬─────────────────────────────┘
                           │
             Sub-15ms Live Environment Probe
                           │
      ┌────────────────────┼────────────────────┐
      ▼                    ▼                    ▼
[Git Working Tree]  [Plugin Manifest]   [Project Adoption]
 Branch: feat/login  Drift: 0 files       AGENTS.md: Valid SHA
 2 files modified    (SHA256 verified)    (SSOT verified)
      │                    │                    │
      └────────────────────┼────────────────────┘
                           ▼
          Canonical State Σ0 Injected at Turn 0
             (Zero Wasted Turns / Zero Lag)
```

Because the agent receives the exact disk state before it generates even a single line of thought, it achieves **0-step recovery**: zero hallucination turns, zero token waste upon state synchronization.

---

## 4. Delivery Architecture: Automated Hooks vs Prompt Directives

How does `ce-ai workflow resume` actually reach the agent at session start without relying on human or LLM memory?

`ce-ai` implements a multi-tier delivery architecture:

1. **Native Shell Lifecycle Hook (Automated — Claude Code):**
   When `ce-ai init-prj` adopts a project containing a `.claude/` directory, it automatically and non-destructively injects a `SessionStart` hook into `.claude/settings.json`:
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
   Claude Code automatically executes this hook at session startup and streams its output directly into the agent's context window.

2. **Native Plugin Lifecycle Event & Compaction Hook (Automated — OpenCode):**
   When `ce-ai install --harness opencode` or `ce-ai sync` runs, it installs the canonical plugin loader at `~/.config/opencode/compound-engineering/plugins/compound-engineering.js` and registers it in `opencode.json`. The plugin subscribes directly to OpenCode's internal lifecycle hooks:
   - **`session.created`**: Upon initialization of every session, runs `ce-ai workflow resume` in the project directory and silently injects live `RepoState` into the session via `client.session.prompt` with `{ noReply: true }`.
   - **`experimental.session.compacting`**: Injects live `RepoState` directly into `output.context`, ensuring canonical drift status survives context compaction.

3. **Native Repository-Level Hook & Context Injection (Automated — GitHub Copilot CLI):**
   When `ce-ai init-prj` adopts a project containing a `.github/` directory, it automatically and non-destructively injects a `sessionStart` command hook into `.github/hooks/hooks.json`:
   ```json
   {
     "version": 1,
     "hooks": {
       "sessionStart": [
         {
           "type": "command",
           "bash": "ce-ai workflow resume --json",
           "powershell": "ce-ai workflow resume --json",
           "timeoutSec": 15
         }
       ]
     }
   }
   ```
   GitHub Copilot CLI executes this hook on startup. `ce-ai workflow resume --json` outputs an `additionalContext` string alongside structured state metadata, which Copilot CLI ingests and injects directly into the agent's prompt context before the first user response.

4. **Native Project TOML Hook & Compaction Resilience (Automated — OpenAI Codex CLI):**
   When `ce-ai init-prj` adopts a project containing a `.codex/` directory, it automatically and non-destructively injects a `SessionStart` command hook into `.codex/config.toml`:
   ```toml
   [[hooks.SessionStart]]
   matcher = "startup|resume|compact"

   [[hooks.SessionStart.hooks]]
   type = "command"
   command = "ce-ai workflow resume"
   statusMessage = "Loading ce-ai workflow state"
   ```
   OpenAI Codex CLI executes this hook on startup, resume, and immediately after mid-turn session compaction (`source: "compact"`), streaming `stdout` directly into the agent's developer context window.

5. **Native In-Process Extension & Prompt Injection (Automated — Pi Coding Agent):**
   When `ce-ai init-prj` adopts a project containing a `.pi/` directory, it automatically deploys a TypeScript lifecycle extension at `.pi/extensions/compound-engineering.ts`:
   ```typescript
   import { execSync } from "node:child_process";

   export default function (pi: any) {
     let sessionInitialized = false;

     pi.on("session_start", async () => {
       sessionInitialized = false;
     });

     pi.on("before_agent_start", async (event: any, ctx: any) => {
       if (!sessionInitialized) {
         sessionInitialized = true;
         try {
           const stdout = execSync("ce-ai workflow resume", {
             cwd: ctx?.cwd || process.cwd(),
             encoding: "utf-8",
             timeout: 5000,
           });
           if (stdout && stdout.trim()) {
             return {
               systemPrompt: `${event.systemPrompt}\n\n<!-- CE-AI MANAGED REPOSTATE -->\n${stdout.trim()}`,
             };
           }
         } catch {
           // Fail-open
         }
       }
     });
   }
   ```
   Pi automatically discovers and executes this extension with its built-in `jiti` runtime, executing `ce-ai workflow resume` on Turn-0 and injecting live `RepoState` directly into `systemPrompt` before the agent starts processing.

6. **Universal Turn-0 Directive (Enforced — Other Prompt-Driven Harnesses):**
   For harnesses that do not yet provide native shell lifecycle hooks or plugin runtimes (Cursor, Kimi, Grok), `ce-ai init-prj` injects a mandatory Turn-0 directive into the managed block of `AGENTS.md`:
   > *"At the start of EVERY new session or after context compaction, before running any task or reading historical chat assumptions, the AI agent MUST run `ce-ai workflow resume`."*

7. **Checkpoint Verification Gate:**
   When an agent records progress via `ce-ai workflow checkpoint`, `ce-ai` automatically probes `RepoState` and surfaces non-blocking warnings if drift or modified files are detected.

---

## 5. What It Looks Like in Practice

### Interactive Developer Output
When you run `ce-ai workflow resume` in your terminal:

```text
workflow: resuming execution from latest checkpoint...
== [Workflow FSM & Progress Recovery Status] ==
  current phase: Stage 4: Work/TDD (work)
  active subtask: Implementing auth handler
  active feature: user-authentication
  last updated: 2026-09-02T03:15:00Z

== [Environment State & Drift Status] ==
  git branch: feat/login (HEAD: 7a8b9c0)
  working tree: 2 modified files (src/auth.rs, src/main.rs)
  manifest integrity: clean (0 drifted files)
  adoption block: ok (SHA256 verified)

== [Context Re-hydration: user-authentication] ==
  spec location: openspec/changes/user-authentication
  has proposal: true
  has spec: true
  has tasks: true
  tasks progress: 2/5 completed ([x])

workflow: re-hydrated context successfully. Proceeding with active task.
```

### Machine-Readable JSON Output for Agents
Autonomous harnesses run `ce-ai workflow resume --json` to receive structured data:

```json
{
  "workflow": {
    "stage": "worktdd",
    "task": "Implementing auth handler",
    "feature_name": "user-authentication"
  },
  "repo_state": {
    "git_branch": "feat/login",
    "head_sha": "7a8b9c0",
    "is_git_clean": false,
    "modified_files": ["src/auth.rs", "src/main.rs"],
    "manifest_drift_count": 0,
    "adoption_status": "ok",
    "openspec_context": {
      "feature": "user-authentication",
      "completed_tasks": 2,
      "total_tasks": 5
    }
  }
}
```

---

## 5. Non-Blocking Guidance on Drift

What happens if managed files (like skills or plugin scripts) were edited outside `ce-ai`?

`ce-ai` will **never crash or halt your work**. Instead, it surfaces clear, non-blocking guidance:

```text
  manifest integrity: ! 2 files modified outside ce-ai
  ! Warning: Drift detected in managed files. Run 'ce-ai sync' to reconcile.
```

If you see this warning:
1. You can continue working safely.
2. When ready, simply run `ce-ai sync` to cleanly reconcile the drifted files against their cryptographic SHA256 hashes.

---

## 6. Summary Checklist for Newcomers

| Concept | What it Means |
| :--- | :--- |
| **Environment Drift** | Files or branches changed outside the AI's chat context. |
| **Turn-0 Recovery** | Syncing disk state *before* the AI plans its next move. |
| **`RepoState`** | The structured snapshot of Git, Manifest, and OpenSpec progress. |
| **Deterministic SHA256** | File integrity is proven by exact hashes, never guessed by timestamps. |
| **Non-Blocking** | Drift warnings alert you without breaking or crashing your workflow. |

---

*Related reading:*
- [Quick Start Workflow Guide](quick-start-workflow-guide.md) — Walk through your first complete 7-stage cycle.
- [FSM & Checkpoints Masterclass](fsm-and-checkpoints-explained.md) — Understand stage transitions and savegames.
- [Sync & Upgrade Mechanisms](sync-and-upgrade-mechanisms.md) — Learn how SHA256 reconciliation works under the hood.
