# Exploration: Companion MCP Server Invocation Arguments

## Context & Background
`ce-ai` provisions companion MCP servers (`codegraph`, `engram`, `context7`, `rtk`) into supported AI harnesses (OpenCode, Claude Code, Cursor, GitHub Copilot CLI, OpenAI Codex CLI, Grok, Kimi Code, Google Antigravity, and Fx).

When companion auto-registration was consolidated in v1.20.3 (`src/harness/registration.rs`) and v1.41.0 (`src/opencode/config.rs`, `src/harness/custom.rs`), the arguments passed for `codegraph` and `engram` were specified as:
- `codegraph`: `&["mcp"]`
- `engram`: `&["serve"]`

## Investigation of External Tool Contracts

### 1. CodeGraph
Running `codegraph --help` on the current version reveals:
```
Commands:
  init [options] [path]          Initialize CodeGraph
  index [options] [path]         Rebuild full index
  serve [options]                Start CodeGraph as an MCP server for AI assistants
  ...
```
There is no `mcp` subcommand. Running `codegraph mcp` fails with:
```
error: unknown command 'mcp'
```
Checking `codegraph serve --help`:
```
Usage: codegraph serve [options]
Start CodeGraph as an MCP server for AI assistants
Options:
  --mcp              Run as MCP server (stdio transport)
```
And running `codegraph install --print-config claude` / `codegraph install --print-config opencode`:
```json
{
  "command": "codegraph",
  "args": ["serve", "--mcp"]
}
```
**Conclusion for CodeGraph**: The stdio MCP command is `codegraph` with arguments `["serve", "--mcp"]`.

### 2. Engram
Running `engram --help` reveals:
```
Commands:
  serve [port]       Start HTTP API server (default: 7437)
  mcp [--tools=PROFILE] [--project NAME]
                     Start MCP server (stdio transport, for any AI agent)
                       Profiles: agent (15 tools), admin (4 tools), all (default, 19)
                       Combine: --tools=agent,admin or pick individual tools
                       Example: engram mcp --tools=agent
```
And its recommended MCP configuration:
```json
{
  "mcp": {
    "engram": {
      "type": "stdio",
      "command": "engram",
      "args": ["mcp", "--tools=agent"]
    }
  }
}
```
`engram serve` starts an HTTP server on port 7437 and fails with `listen tcp 127.0.0.1:7437: bind: address already in use` when launched under an AI agent, or fails stdio transport handshake.
**Conclusion for Engram**: The stdio MCP command is `engram` with arguments `["mcp", "--tools=agent"]`.

## Evaluated Approaches
- **Option A (Centralized Constants & Slice Replacement)**:
  Update the argument slices in `RegistrationSpec::register_companions`, `opencode::config::register_companions`, `harness::custom::register_companions`, and `tools::mcp_spec_for_tool`. Update all tests asserting the old arguments.
  - *Pros*: Completely fixes the root cause across all harnesses; immediately heals configurations on `ce-ai sync` or `upgrade`; 100% type-safe and consistent.
  - *Cons*: Modifies multiple unit test fixtures that asserted the old strings.
- **Option B (Heuristic Fallback)**:
  Only change for Claude Code and OpenCode.
  - *Cons*: Leaves Cursor, Kimi, Copilot, Codex, Grok, Antigravity, and Fx broken. Violates architectural consistency.

**Decision**: Option A.
