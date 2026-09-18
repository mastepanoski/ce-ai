# Proposal: Fix Companion MCP Server Invocation Arguments for CodeGraph and Engram

## Problem Statement
When `ce-ai` executes companion MCP registration during `install`, `sync`, `upgrade`, or `tools install`, it writes stdio MCP configuration entries for companion tools into AI harness configuration files (`~/.claude/settings.json`, `~/.cursor/mcp.json`, `~/.config/opencode/opencode.json`, `~/.kimi-code/mcp.json`, `~/.copilot/mcp-config.json`, etc.).

Currently, `ce-ai` hardcodes invalid commands and arguments for two core companion tools:
1. **CodeGraph**: Registered with `command: "codegraph"`, `args: ["mcp"]`.
   - CodeGraph CLI does not have an `mcp` subcommand (`codegraph mcp` exits with `error: unknown command 'mcp'`).
   - CodeGraph's actual stdio MCP server command is `codegraph serve --mcp`.
2. **Engram**: Registered with `command: "engram"`, `args: ["serve"]`.
   - `engram serve` starts an HTTP server on TCP port `127.0.0.1:7437`. It does not speak JSON-RPC over stdio.
   - When launched as a stdio server by an agent harness (e.g. Claude Code or OpenCode), `engram serve` attempts to bind TCP 7437. Because the background engram daemon is typically already running, it immediately terminates with `listen tcp 127.0.0.1:7437: bind: address already in use`. Even if port 7437 were unbound, the process would hang and fail the stdio MCP handshake.
   - Engram's actual stdio MCP server command is `engram mcp --tools=agent`.

When developers run `ce-ai upgrade`, `ce-ai sync`, or `ce-ai install`, `ce-ai` overwrites any working MCP server entries with these invalid arguments, breaking companion integration for all registered harnesses.

## In-Scope
1. **Centralized Registration Spec Correction**: Update `RegistrationSpec::register_companions` in `src/harness/registration.rs` to register:
   - `codegraph` with `args: ["serve", "--mcp"]`
   - `engram` with `args: ["mcp", "--tools=agent"]`
2. **OpenCode Companion Registration Correction**: Update `crate::opencode::config::register_companions` in `src/opencode/config.rs` to register:
   - `codegraph` with `args: ["serve", "--mcp"]`
   - `engram` with `args: ["mcp", "--tools=agent"]`
3. **Custom Harness Companion Registration Correction**: Update `crate::harness::custom::register_companions` in `src/harness/custom.rs` to register:
   - `codegraph` with `args: ["serve", "--mcp"]`
   - `engram` with `args: ["mcp", "--tools=agent"]`
4. **Tools CLI MCP Spec Correction**: Update `mcp_spec_for_tool` in `src/commands/tools.rs` to map:
   - `"codegraph"` -> `("codegraph", &["serve", "--mcp"])`
   - `"engram"` -> `("engram", &["mcp", "--tools=agent"])`
5. **Test Suite Alignment**: Update all unit tests across harness adapters (`claude`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`, `custom`, `opencode`, `registration`, `tools`) to assert the correct stdio MCP arguments.
6. **Empirical Verification**: Run `cargo test`, `cargo clippy`, and `cargo fmt`.

## Out-of-Scope
1. Modifying external companion binary internals (`codegraph` or `engram`).
2. Altering other companion tools (`context7` or `rtk` hooks).
3. Modifying network download logic for companion binaries.

## Risk Evaluation & Mitigation
- **Risk (Regressions in Harness Parsing)**: Different harnesses serialize MCP configs differently (JSON, TOML, `args` array vs embedded).
  - *Mitigation*: The change only affects the `args` array slice passed to the existing typed registrars. No schema or serialization format changes.
- **Risk (Backward Compatibility)**: Existing user configs with invalid args will be automatically healed on the next `ce-ai sync` or `ce-ai upgrade`.
