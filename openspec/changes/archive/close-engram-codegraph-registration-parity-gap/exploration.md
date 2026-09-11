# Exploration: Companion Registration Parity Gap

## 1. Current State Investigation
In `src/harness/registration.rs`:
```rust
pub(crate) fn registration_spec(kind: HarnessKind) -> Option<RegistrationSpec> {
    let native = |reg: McpRegistrar| RegistrationSpec {
        register_mcp: Some(reg),
    };
    Some(match kind {
        HarnessKind::Cursor => RegistrationSpec {
            register_mcp: Some(crate::harness::cursor::register_cursor_mcp_server),
        },
        HarnessKind::Claude => native(crate::harness::claude::register_claude_mcp_server),
        HarnessKind::Codex => native(crate::harness::codex::register_codex_mcp_server),
        HarnessKind::Copilot => native(crate::harness::copilot::register_copilot_mcp_server),
        HarnessKind::Grok => native(crate::harness::grok::register_grok_mcp_server),
        HarnessKind::Kimi => native(crate::harness::kimi::register_kimi_mcp_server),
        HarnessKind::Agy => native(crate::harness::agy::register_agy_mcp_server),
        HarnessKind::Fx => native(crate::harness::fx::register_fx_mcp_server),
        HarnessKind::Pi => RegistrationSpec { register_mcp: None },
        HarnessKind::Custom | HarnessKind::Opencode | HarnessKind::Deepseek => return None,
    })
}
```

### Analysis of the 4 Harnesses

#### A. OpenCode
- **Current Behavior**: In `install.rs` and `sync.rs`, OpenCode is handled in a dedicated arm that executes `ensure_plugin_and_skills(&target_config, ...)`. It does not execute `RegistrationSpec::register_companions`.
- **Existing Capabilities**: `src/opencode/config.rs` has `register_mcp_server(config_path, tool_name, server_def)`. When users run `ce-ai tools install engram`, it calls this function directly.
- **Resolution**: Provide `crate::opencode::config::register_companions(&target_config)` calling `register_mcp_server` for `codegraph` (`["mcp"]`) and `engram` (`["serve"]`), and invoke it in OpenCode's dedicated arms in both `install.rs` and `sync.rs`.

#### B. Custom
- **Current Behavior**: Custom harness has a snapshot-driven layout in `CustomHarnessConfig` (`plugins_dir`, `skills_dir`, `rules_file`). It does not register MCP servers.
- **Resolution**: Add an optional `mcp_file: Option<PathBuf>` to `CustomHarnessConfig` and `CustomConfigFlags` (with `--mcp-file` CLI flag). When `mcp_file` is specified, `ce-ai install` and `sync` call `crate::harness::custom::register_custom_mcp_server`, registering `codegraph` and `engram` under standard `mcpServers`. If absent, MCP registration is skipped cleanly.

#### C. Deepseek
- **Current Behavior**: `Deepseek` is marked as preview/unsupported across `ce-ai`. `ce-ai install --harness deepseek` returns `CeError::Usage` indicating that DeepSeek uses YAML patch layers under `~/.dsh`.
- **Resolution**: Deepseek remains `return None` in `registration_spec`. Document this architectural decision explicitly in doc comments and tests, removing the ambiguity of a silent omission.

#### D. Pi
- **Current Behavior**: Pi is `RegistrationSpec { register_mcp: None }`. The code comments state `// Pi is No-MCP by design (objective 8): skills tree only.`
- **Resolution**: Pi continues to return `register_mcp: None`. Its delivery mechanism is via CLI binaries available on `PATH` (`engram`, `codegraph`) coupled with the skills tree (`~/.pi/agent/skills/`). In `doctor.rs` and `tools.rs`, explicitly surface that Pi uses CLI-based skills tree integration, avoiding false "missing MCP" diagnostics.
