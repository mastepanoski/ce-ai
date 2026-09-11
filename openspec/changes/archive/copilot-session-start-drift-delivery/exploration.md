# Exploration: GitHub Copilot CLI Hook Architecture & Context Delivery

## 1. Technical Investigation of Copilot CLI Hooks

According to official GitHub documentation and community empirical verification:
- **Configuration Locations:**
  - Repository-level: `.github/hooks/*.json` (standard convention: `.github/hooks/hooks.json`).
  - Personal/user-level: `~/.copilot/hooks/*.json`.
- **Top-Level File Structure:**
  ```json
  {
    "version": 1,
    "hooks": {
      "sessionStart": [ ... ]
    }
  }
  ```
- **Hook Command Entry Schema:**
  ```json
  {
    "type": "command",
    "bash": "ce-ai workflow resume --json",
    "powershell": "ce-ai workflow resume --json",
    "timeoutSec": 15
  }
  ```
- **Context Injection Contract:**
  When Copilot CLI fires `sessionStart`, it executes the command. If the process exits with status 0 and outputs a JSON object containing an `additionalContext` string, Copilot CLI extracts that string and adds it to the agent's conversation context before the first user turn. Non-JSON stdout is discarded.

## 2. Output Schema Adaptation in `workflow.rs`

Currently `ce-ai workflow resume --json` produces:
```json
{
  "workflow": { ... },
  "repo_state": { ... },
  "openspec_context": { ... }
}
```
By adding `"additionalContext": resume_lines(ctx)?.join("\n")`, `ce-ai` outputs both machine-readable state objects AND the formatted markdown string Copilot CLI expects, with zero breaking changes for existing consumers.

## 3. Surgical Lifecycle Tradeoffs

- When `hooks.json` already exists: Merge the hook into `hooks.sessionStart` array. If an entry with matching command already exists, do not duplicate.
- When `deinit-prj` runs: Filter out the hook entry. If `sessionStart` becomes empty, remove the key. If `hooks` becomes empty, remove `hooks`. If the root object becomes `{}` or only has `"version": 1`, remove `hooks.json` and prune `.github/hooks` if empty.
