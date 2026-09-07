# Exploration: Cursor sessionStart Lifecycle Hook Integration

## Technical Investigation

### 1. Cursor Hook Architecture & Lifecycle Points
- Cursor supports lifecycle hooks configured via `.cursor/hooks.json` (project-level) or `~/.cursor/hooks.json` (global).
- The `sessionStart` hook is triggered whenever a new composer conversation is initialized in both Cursor IDE and Cursor CLI (supported as of v0.45+).
- Hooks execute as subprocesses communicating via `stdio` using JSON.

### 2. Context Injection Interface
- On `sessionStart`, Cursor passes session metadata via `stdin` (`session_id`, `is_background_agent`, `composer_mode`).
- The script returns a JSON object via `stdout`.
- The documented output schema accepts:
  - `additional_context`: string to add to the conversation's initial system context.
  - `env`: key-value map of environment variables available to subsequent hooks.
- Currently `ce-ai workflow resume --json` outputs `additionalContext` (camelCase for Copilot) and `hookSpecificOutput` (for Codex). Adding `additional_context` (snake_case) fulfills Cursor's schema directly without breaking Copilot or Codex.

### 3. File Location & De-Adoption
- File path: `<project>/.cursor/hooks.json`.
- Schema:
  ```json
  {
    "version": 1,
    "hooks": {
      "sessionStart": [
        {
          "command": "ce-ai workflow resume --json"
        }
      ]
    }
  }
  ```
- When de-adopting, if `.cursor/hooks.json` only contains our managed `sessionStart` hook and no other hooks or properties, the file should be cleanly deleted.

## Alternatives Evaluated
1. **Plain text stdout:** Cursor's documentation states that command hooks receive JSON input and return JSON output. Using `ce-ai workflow resume --json` is required for reliable parsing by Cursor.
2. **Global `~/.cursor/hooks.json` only:** Project-level isolation in `.cursor/hooks.json` ensures that individual repos remain self-contained without polluting user-wide settings.
