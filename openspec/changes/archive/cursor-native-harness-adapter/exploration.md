# Exploration: Cursor Native Harness Adapter

## 1. Technical Investigation

### Current Behavior vs Cursor Official Specification
- **Current `ce-ai` Behavior**: `install --harness cursor` writes `~/.cursor/mcp.json` with keys `{"plugin": [...], "skills": {"paths": [...]}}`. Cursor ignores these keys completely.
- **Cursor Official Spec (August 2026)**:
  - Config location: `~/.cursor/mcp.json`
  - Schema:
    ```json
    {
      "mcpServers": {
        "codegraph": {
          "type": "stdio",
          "command": "codegraph",
          "args": ["mcp"],
          "env": {}
        }
      }
    }
    ```
  - Rules location: `.cursor/rules/<name>.mdc` with frontmatter:
    ```markdown
    ---
    description: "Compound Engineering Agent Directives"
    globs: "*"
    alwaysApply: true
    ---
    ```

## 2. Options Evaluated

1. **Option A: Write OpenCode schema everywhere (Status Quo)**:
   - *Pros*: Simple, zero adapter code.
   - *Cons*: Fails silently on Cursor, zero MCP tools or skills loaded. Unacceptable.
2. **Option B: Native Cursor Adapter (Selected)**:
   - *Pros*: Full compatibility with Cursor, native `mcpServers` stdio schema, structured JSON merge, clean lifecycle.
   - *Cons*: Requires per-harness adapter implementation in `src/harness/cursor.rs`.
