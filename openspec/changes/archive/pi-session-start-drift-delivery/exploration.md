# Exploration: Pi Coding Agent Extension Architecture & Lifecycle Events

## 1. Extension Discovery & Execution
- **Path:** Project-local `.pi/extensions/*.ts` (or `~/.pi/agent/extensions/*.ts`).
- **Loader:** Pi uses an internal loader (powered by `jiti`) that dynamically loads, transpiles, and executes `.ts` or `.js` modules. No pre-compilation step or `package.json` build workflow is required.
- **Exports:** Module exports a default function receiving `ExtensionAPI`:
  ```typescript
  export default function (pi: ExtensionAPI) { ... }
  ```

## 2. Lifecycle Events
- **`session_start`**:
  - Fires when a session starts or switches (via `/resume`, `/new`, `/fork`).
  - Does *not* directly inject context into the prompt; serves to reset session state.
- **`before_agent_start`**:
  - Fires immediately before each agent turn (prompt evaluation).
  - Can return `{ systemPrompt: ... }` to inject custom instructions or context directly into the LLM system prompt.
  - Receives `(event, ctx)` where `ctx.cwd` provides the project root directory.

## 3. Turn-0 Coordination Design
To avoid running `ce-ai workflow resume` redundantly on every single message turn:
- Maintain a local `sessionInitialized` flag in the extension module.
- `session_start` resets `sessionInitialized = false`.
- The first `before_agent_start` invocation checks `if (!sessionInitialized)`, executes `ce-ai workflow resume`, appends the output to `event.systemPrompt`, and sets `sessionInitialized = true`.
- Subsequent user turns within the same session proceed without spawning child processes.
