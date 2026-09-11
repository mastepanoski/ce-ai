# Exploration: MCP-Configured Companion Tools & Skills Detection

## Problem Analysis
In `ce-ai`, companion tools and skills belong to two distribution categories:
1. **Binary CLI executables**: tools like `codegraph`, `engram`, and `rtk` that install an executable on the system `PATH`. These respond to `<tool> --version`.
2. **MCP server wrappers**: tools like `context7` (registered via `npx -y @upstash/context7-mcp@latest`) and skills like `sequential-thinking` (registered via `npx -y @modelcontextprotocol/server-sequential-thinking`). These are never installed as global binaries on `PATH`. Instead, their lifecycle is managed via an `mcpServers` object in harness configurations:
   - OpenCode: `opencode.json` (`mcpServers.<name>`)
   - Cursor: `~/.cursor/mcp.json` (`mcpServers.<name>`)
   - Claude: `~/.claude.json` / `claude_desktop_config.json` (`mcpServers.<name>`)
   - Copilot: `~/.copilot/mcp-config.json` (`mcpServers.<name>`)
   - Kimi: `~/.kimi-code/mcp.json` (`mcpServers.<name>`)
   - Antigravity: `~/.gemini/config/mcp_config.json` (`mcpServers.<name>`)

Currently, both `doctor.rs` and `tools.rs` rely solely on:
```rust
let installed = extract_tool_version(name);
let freshness = evaluate_freshness(installed.as_deref(), &info.latest_version);
```
Where `extract_tool_version` runs `Command::new(tool_name).arg("--version")`.
Because `Command::new("context7")` fails with `NotFound`, `evaluate_freshness(None, ...)` evaluates to `FreshnessStatus::Missing`.

Similarly, `doctor.rs` iterates `registry.skills` unconditionally:
```rust
for (name, skill) in &registry.skills {
    println!("doctor-info: skill-suggestion: {} (run '{}')", name, skill.resolve_cmd);
}
```
Without probing whether the user has already registered the MCP server or skill.

## Evaluated Options

### Option 1: Ad-hoc check in `doctor.rs` only
- **Pros**: Quick to write.
- **Cons**: Duplicates logic; leaves `ce-ai tools status` broken, continuing to print `❌ context7 [MCP Server] : not found`.

### Option 2: Shared detection in `src/source/tools_registry.rs` (Selected)
- **Pros**:
  - Single source of truth for companion tool and skill freshness across all CLI entry points (`doctor`, `tools status`).
  - Probes both system `PATH` and active harness MCP configurations (`mcpServers`).
  - Handles naming variations (hyphenated vs non-hyphenated like `sequential-thinking` vs `sequentialthinking`).
  - Inspects command and argument strings inside server definitions for robust matching.
- **Cons**: Slightly broader test surface.
