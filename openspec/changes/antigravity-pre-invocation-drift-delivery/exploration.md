# Exploration: Antigravity PreInvocation Turn-0 Drift Delivery

## Technical Investigation

### 1. Official Hook Architecture (https://antigravity.google/docs/hooks/)
- Supported events: `PreToolUse`, `PostToolUse`, `PreInvocation`, `PostInvocation`, `Stop`.
- Configured in `hooks.json` under `.agents/` (workspace) or `~/.gemini/config/` (global).
- Top-level schema maps hook group names to event configurations:
  ```json
  {
    "compound-engineering": {
      "PreInvocation": [
        {
          "type": "command",
          "command": "ce-ai workflow resume --pre-invocation"
        }
      ]
    }
  }
  ```
- Matcher is ignored for `PreInvocation`.

### 2. Input / Output Contract
- **Input (`stdin`)**: JSON containing:
  - `invocationNum`: 0-indexed sequence number of model invocation.
  - `conversationId`: unique conversation/session identifier.
  - `workspacePaths`: array of workspace root paths.
  - `transcriptPath`: path to session transcript.
- **Output (`stdout`)**: JSON containing:
  - `injectSteps`: array of step objects, supporting `ephemeralMessage` (transient system message).
  - Or empty object `{}` when no injection is required.

### 3. Session Deduplication Strategy
- Because `PreInvocation` fires before every model turn, injecting on every invocation would flood the context with duplicate `RepoState` blocks.
- Using `conversationId`, `ce-ai workflow resume --pre-invocation` checks for the existence of `std::env::temp_dir().join(format!("ce-ai-agy-session-{conversationId}.marker"))`.
- Turn 0: Marker absent -> creates marker file -> emits `{"injectSteps": [{"ephemeralMessage": "<resume text>"}]}`.
- Turn 1+: Marker present -> emits `{}` (no-op).

### 4. Alternatives Evaluated
- **External shell/bash script wrapper:** Creating a shell script wrapper requires managing platform differences (POSIX `.sh` vs Windows PowerShell/batch). Integrating `--pre-invocation` directly into the `ce-ai` binary guarantees native cross-platform execution on macOS, Linux, and Windows with zero extra files.
