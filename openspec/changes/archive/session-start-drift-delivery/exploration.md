# Exploration: Turn-0 Delivery Mechanisms & Lifecycle Hooks

## Technical Investigation

### 1. Claude Code Lifecycle Hooks
Claude Code provides official support for lifecycle hooks configured in `.claude/settings.json` (project level) or `~/.claude/settings.json` (global level).
- **Hook Event:** `SessionStart` fires at the start of every new session, resume, or clear.
- **Payload Schema:**
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
- **Context Injection Behavior:** Claude Code executes the shell command and automatically streams its `stdout` directly into the agent's context window as system context before the user's first prompt or agent action.
- **Idempotency & Merging:** If `.claude/settings.json` already exists with other keys (e.g. `mcpServers`, other hooks, permissions), `ce-ai` must parse it into a generic JSON object, check if `command == "ce-ai workflow resume"` already exists in `hooks.SessionStart`, and append it without clobbering other settings.

### 2. Universal Harness Support via `AGENTS.md`
Harnesses such as Cursor, GitHub Copilot, Codex, Kimi, Grok, and Pi do not currently expose standardized, project-scoped shell lifecycle hooks that execute shell commands on startup.
- **Baseline Invariant:** In these harnesses, the agent's behavior is guided by instruction files (`AGENTS.md`, `.cursorrules`, `.github/copilot-instructions.md`).
- **Turn-0 Directive:** Elevating `workflow resume` from an assumed practice to an explicit Turn-0 directive in the managed block guarantees that instruction-following LLMs execute `workflow resume` before taking any exploratory or generative action.
- **Template Evolution:** Changing `render_block_content` necessitates bumping `BLOCK_VERSION` 3 → 4. The existing adoption engine in `init_prj.rs`, `doctor.rs`, and `status.rs` seamlessly identifies `v=3` blocks as `StaleVersion` and directs users to upgrade.

### 3. Checkpoint & Health Probing
- When an agent calls `ce-ai workflow checkpoint <stage> <task>`, running `probe_repo_state()` provides immediate feedback on whether the working tree has drifted since the last resume.
- In `ce-ai doctor`, checking `.claude/settings.json` provides an empirical audit of whether the native hook is actively configured.

## Evaluated Alternatives & Tradeoffs

| Approach | Feasibility | Overhead | Assessment |
| :--- | :--- | :--- | :--- |
| **A. Shell Wrapper (`alias claude="ce-ai workflow resume && claude"`)** | Low | High | **Rejected:** Fragile, pollutes user shell configs, fails in GUI/IDE launches. |
| **B. Background Watcher Daemon** | Medium | High | **Rejected:** Burns CPU/memory, introduces race conditions and OS-specific daemon managers. |
| **C. Defense-in-Depth (Native Hook + Textual Directive + Gate)** | High | Minimal (<15ms) | **Selected:** Cleanest separation of concerns, native to Claude Code, 100% portable across all other harnesses. |
