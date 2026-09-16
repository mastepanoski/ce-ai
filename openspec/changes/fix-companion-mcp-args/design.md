# Design: Companion MCP Server Invocation Arguments

## Architecture Overview
`ce-ai` coordinates companion tools across AI agent harnesses. When registering companion MCP servers:
- CodeGraph must run in stdio MCP mode: `codegraph serve --mcp`
- Engram must run in stdio MCP mode: `engram mcp --tools=agent`

## Constants and Registration Sites

### 1. `src/harness/registration.rs`
In `RegistrationSpec::register_companions`:
```rust
register(target_config, "codegraph", "codegraph", &["serve", "--mcp"], &env).map_err(wrap_io)?;
register(target_config, "engram", "engram", &["mcp", "--tools=agent"], &env).map_err(wrap_io)?;
```

### 2. `src/opencode/config.rs`
In `register_companions`:
```rust
register_mcp_server(
    config_path,
    "codegraph",
    serde_json::json!({ "command": "codegraph", "args": ["serve", "--mcp"] }),
)?;
register_mcp_server(
    config_path,
    "engram",
    serde_json::json!({ "command": "engram", "args": ["mcp", "--tools=agent"] }),
)?;
```

### 3. `src/harness/custom.rs`
In `register_companions`:
```rust
register_custom_mcp_server(mcp_file, "codegraph", "codegraph", &["serve", "--mcp"], &env)?;
register_custom_mcp_server(mcp_file, "engram", "engram", &["mcp", "--tools=agent"], &env)?;
```

### 4. `src/commands/tools.rs`
In `mcp_spec_for_tool`:
```rust
fn mcp_spec_for_tool(tool: &str) -> Option<(&'static str, &'static [&'static str])> {
    match tool {
        "context7" => Some(("npx", &["-y", "@upstash/context7-mcp@latest"])),
        "engram" => Some(("engram", &["mcp", "--tools=agent"])),
        "rtk" => Some(("rtk", &["mcp"])),
        "codegraph" => Some(("codegraph", &["serve", "--mcp"])),
        _ => None,
    }
}
```

## Affected Test Suites
The following test suites assert companion registration arguments and must be updated to match the new, correct arguments:
1. `src/harness/tests/registration.rs`
2. `src/opencode/tests/config.rs`
3. `src/harness/tests/custom.rs`
4. `src/harness/tests/claude.rs`
5. `src/harness/tests/cursor.rs`
6. `src/harness/tests/copilot.rs`
7. `src/harness/tests/codex.rs`
8. `src/harness/tests/grok.rs`
9. `src/harness/tests/kimi.rs`
10. `src/harness/tests/agy.rs`
11. `src/harness/tests/fx.rs`
12. `src/commands/tests/tools.rs` (if applicable)
