# Design: Close engram/codegraph registration parity gap

## 1. Architecture & Component Interaction

```
ce-ai install / sync
  │
  ├─► Native Harnesses (Claude, Codex, Cursor, Copilot, Grok, Kimi, Agy, Fx)
  │     └─► registration_spec(kind) ──► spec.register_companions(&target_config)
  │
  ├─► Opencode (Dedicated Arm)
  │     ├─► ensure_plugin_and_skills(&target_config, ...)
  │     └─► crate::opencode::config::register_companions(&target_config)
  │           ├─► register_mcp_server(target_config, "codegraph", ["mcp"])
  │           └─► register_mcp_server(target_config, "engram", ["serve"])
  │
  ├─► Custom (Dedicated Arm)
  │     ├─► ensure_rules_block / copy plugins & skills
  │     └─► if let Some(mcp_file) = &cfg.mcp_file:
  │           └─► register_custom_mcp_server(mcp_file, "codegraph" / "engram")
  │
  ├─► Pi
  │     └─► registration_spec(Pi) ──► register_mcp: None (No-MCP by design)
  │           └─► Delivery via CLI binaries on PATH + skills tree
  │
  └─► Deepseek
        └─► registration_spec(Deepseek) ──► None (de-scoped: preview dsh YAML layers)
```

## 2. Interface Changes

### OpenCode (`src/opencode/config.rs`)
```rust
/// Registers companion MCP servers (`codegraph`, `engram`) into `opencode.json`.
pub fn register_companions(config_path: &Path) -> Result<(), CeError> {
    register_mcp_server(
        config_path,
        "codegraph",
        serde_json::json!({ "command": "codegraph", "args": ["mcp"] }),
    )?;
    register_mcp_server(
        config_path,
        "engram",
        serde_json::json!({ "command": "engram", "args": ["serve"] }),
    )?;
    Ok(())
}
```

### Custom Harness (`src/harness/custom.rs`)
```rust
pub struct CustomHarnessConfig {
    pub plugins_dir: PathBuf,
    pub skills_dir: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_file: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_file: Option<PathBuf>,
}

pub struct CustomConfigFlags {
    pub plugins_dir: Option<PathBuf>,
    pub skills_dir: Option<PathBuf>,
    pub rules_file: Option<PathBuf>,
    pub mcp_file: Option<PathBuf>,
}
```
Add `pub fn register_custom_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`.

### Registration Characterization Tests (`src/harness/tests/registration.rs`)
Test:
1. `table_driven_harnesses_have_valid_mcp_registrars` (Claude, Codex, Copilot, Cursor, Grok, Kimi, Agy, Fx).
2. `pi_is_explicitly_no_mcp`.
3. `custom_opencode_deepseek_are_explicitly_none_with_documented_rationale`.
4. `register_companions_invokes_codegraph_and_engram`.
5. `register_companions_on_no_mcp_is_noop`.
