# Exploration: OpenAI Codex CLI Hook Architecture & Context Delivery

## 1. Official Documentation Findings

According to `https://developers.openai.com/codex/hooks`:
- **Configuration Locations:**
  - `<repo>/.codex/config.toml` (project-level)
  - `~/.codex/config.toml` (user-level)
  - Also supports `hooks.json`, but `config.toml` is the primary configuration file managed by `ce-ai`.
- **TOML Schema:**
  ```toml
  [[hooks.SessionStart]]
  matcher = "startup|resume|compact"

  [[hooks.SessionStart.hooks]]
  type = "command"
  command = "ce-ai workflow resume"
  statusMessage = "Loading ce-ai workflow state"
  ```
- **Context Delivery Mechanism:**
  - Plain text written to `stdout` is added as extra developer context before the model generates its response.
  - JSON on `stdout` is also supported using `hookSpecificOutput`:
    ```json
    {
      "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "..."
      }
    }
    ```
  - Matching `source: "compact"` ensures that when automatic compaction happens in the middle of a turn, Codex delivers the hook's additional context to the immediate continuation instead of waiting for a later user turn.

## 2. Comparison with Claude Code and OpenCode

| Feature | Claude Code | OpenCode | GitHub Copilot CLI | OpenAI Codex CLI |
| :--- | :--- | :--- | :--- | :--- |
| **Config Location** | `.claude/settings.json` | `opencode.json` + plugin | `.github/hooks/hooks.json` | `.codex/config.toml` |
| **Format** | JSON | JS Plugin / JSON | JSON | TOML |
| **Event Name** | `SessionStart` | `session.created` | `sessionStart` | `SessionStart` |
| **Compaction Event** | Re-run on startup | `experimental.session.compacting` | Discarded | `source: "compact"` in `SessionStart` |
| **Context Channel** | `stdout` (plain text) | `client.session.prompt` | `stdout` (`additionalContext`) | `stdout` (plain text or `hookSpecificOutput`) |
