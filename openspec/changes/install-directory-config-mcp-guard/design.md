# Design: Companion MCP Registration Directory Guard & Contextual Error Wrapping

## Component Architecture

### 1. `RegistrationSpec` Struct Update (`src/harness/registration.rs`)
Add `kind: HarnessKind` field to `RegistrationSpec`:

```rust
#[derive(Clone, Copy)]
pub(crate) struct RegistrationSpec {
    /// The harness kind for this registration spec.
    pub(crate) kind: HarnessKind,
    /// Vendor registrar; `None` for No-MCP harnesses such as pi.
    pub(crate) register_mcp: Option<McpRegistrar>,
}
```

### 2. Guard & Error Wrapping in `register_companions`
Implement the directory check and I/O error wrapping in `RegistrationSpec::register_companions`:

```rust
impl RegistrationSpec {
    pub(crate) fn register_companions(&self, target_config: &Path) -> Result<(), CeError> {
        let Some(register) = self.register_mcp else {
            return Ok(());
        };

        if target_config.exists() && !target_config.is_file() {
            eprintln!(
                "warn: skipping companion MCP registration for {}: '{}' exists but is not a regular config file (expected a JSON/TOML file)",
                self.kind,
                target_config.display()
            );
            return Ok(());
        }

        let env = BTreeMap::new();
        let wrap_io = |err: CeError| match err {
            CeError::Io(e) => {
                let msg = format!("{}: '{}': {e}", self.kind, target_config.display());
                CeError::Io(std::io::Error::new(e.kind(), msg))
            }
            other => other,
        };

        register(target_config, "codegraph", "codegraph", &["mcp"], &env).map_err(wrap_io)?;
        register(target_config, "engram", "engram", &["serve"], &env).map_err(wrap_io)?;
        Ok(())
    }
}
```

### 3. Strategy Table Instantiation (`registration_spec`)
Update `registration_spec(kind: HarnessKind)` to initialize `kind`:

```rust
pub(crate) fn registration_spec(kind: HarnessKind) -> Option<RegistrationSpec> {
    let native = |reg: McpRegistrar| RegistrationSpec {
        kind,
        register_mcp: Some(reg),
    };
    Some(match kind {
        HarnessKind::Cursor => RegistrationSpec {
            kind,
            register_mcp: Some(crate::harness::cursor::register_cursor_mcp_server),
        },
        HarnessKind::Claude => native(crate::harness::claude::register_claude_mcp_server),
        HarnessKind::Codex => native(crate::harness::codex::register_codex_mcp_server),
        HarnessKind::Copilot => native(crate::harness::copilot::register_copilot_mcp_server),
        HarnessKind::Grok => native(crate::harness::grok::register_grok_mcp_server),
        HarnessKind::Kimi => native(crate::harness::kimi::register_kimi_mcp_server),
        HarnessKind::Agy => native(crate::harness::agy::register_agy_mcp_server),
        HarnessKind::Fx => native(crate::harness::fx::register_fx_mcp_server),
        HarnessKind::Pi => RegistrationSpec {
            kind,
            register_mcp: None,
        },
        HarnessKind::Custom | HarnessKind::Opencode | HarnessKind::Deepseek => return None,
    })
}
```

### 4. Diagnostic Warning Format
The warning string format is:
`warn: skipping companion MCP registration for <harness>: '<path>' exists but is not a regular config file (expected a JSON/TOML file)`
Output destination: `stderr` (`eprintln!`).
Exit code: 0 (non-fatal; continues loop).
