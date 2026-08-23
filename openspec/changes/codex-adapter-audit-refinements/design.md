# Design: Codex Adapter Audit Refinements

## 1. Environment Variable Update (`CODEX_HOME`)

In `src/harness/mod.rs`:
```rust
HarnessKind::Codex => std::env::var_os("CODEX_HOME")
    .map(PathBuf::from)
    .unwrap_or_else(|| home_dir.join(".codex")),
```

In `src/harness/codex.rs`:
```rust
impl HarnessAdapter for CodexAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Codex
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("config.toml") {
            return home.to_path_buf();
        }

        if let Ok(config_env) = std::env::var("CODEX_HOME") {
            return PathBuf::from(config_env).join("config.toml");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".codex") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".codex").join("config.toml")
    }
}
```

## 2. Env Table Replacement in `register_codex_mcp_server`

In `src/harness/codex.rs`:
```rust
if !env.is_empty() {
    let mut env_table = toml::Table::new();
    for (k, v) in env {
        env_table.insert(k.clone(), toml::Value::String(v.clone()));
    }
    server_table.insert("env".to_string(), toml::Value::Table(env_table));
} else {
    server_table.remove("env");
}
```

## 3. Legacy Code Cleanup in `src/harness/generic_json.rs`
Remove `HarnessKind::Codex` from `GenericJsonAdapter` and unit tests.

## 4. AGENTS.md Adoption Contract Clarification
- In `src/commands/init_prj.rs`: `ce-ai init-prj` checks if `.codex/` directory exists in the target project. If `.codex/` exists, it adopts project rules in `.codex/AGENTS.md`.
- `AGENTS.md` at project root is `ce-ai`'s own primary directive source and is NOT injected with a managed block to avoid self-referential redundancy.
