# Exploration: Grok Build CLI Turn-0 Drift Delivery Evaluation

## Technical Investigation

### 1. Hook Subsystem Schema
- Grok CLI hook definitions live under `~/.grok/hooks/*.json` or `.grok/hooks/*.json`.
- Event `SessionStart` triggers when a session initializes.
- Process input is delivered over stdin with `{ sessionId, cwd, workspaceRoot }`.

### 2. Stdout Handling Evaluation
- Empirical inspection and documentation confirm that Grok CLI ignores stdout emitted by `SessionStart` hooks.
- Unlike Claude Code or Cursor, fields such as `additionalContext` or `additional_context` are not ingested into the conversation history.
- The hook lifecycle in Grok CLI is strictly designed for environment initialization, telemetry, or blocking security gates (exit code 2).

### 3. Alternative Approaches Evaluated
- **PreToolUse Hook:** Executes on each tool call rather than Turn-0. Does not prevent initial prompt hallucinations.
- **Text Directives:** `ce-ai init-prj` already injects mandatory Turn-0 execution directives into `AGENTS.md` and `.grok/rules/compound-engineering.md`. This remains the most reliable mechanism.
