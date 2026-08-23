# Design: Copilot Adapter Audit Refinements

## 1. Clean Env Object Handling (`src/harness/copilot.rs`)

```rust
server_entry.env = env.clone();
```

With `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]` on `CopilotMcpServer.env`, passing a non-empty `env` map serializes `{ "env": { ... } }`, while passing an empty `env` map omits/removes the `env` key from `~/.copilot/mcp-config.json`.

## 2. Skills Removal Warning Emission (`src/commands/uninstall.rs`)

```rust
let skills_dir = config_dir.join("skills");
if skills_dir.exists() {
    if let Err(e) = std::fs::remove_dir_all(&skills_dir) {
        if !ctx.quiet {
            eprintln!(
                "warning: failed to clean skills directory at {}: {e}",
                skills_dir.display()
            );
        }
    }
}
```

## 3. Explicit `COPILOT_CONFIG_DIR` Documentation
Update `openspec/changes/copilot-native-harness-adapter/design.md` and `openspec/changes/copilot-native-harness-adapter/spec.md` to note `COPILOT_CONFIG_DIR` is `ce-ai`'s environment variable convention for isolation during tests and multi-profile setups.
