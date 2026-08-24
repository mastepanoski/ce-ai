# Design: Grok Native Harness Adapter (Issue #176)

## 1. Structural Data Models & Adapter (`src/harness/grok.rs`)

```rust
pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct GrokAdapter;

impl HarnessAdapter for GrokAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Grok
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("config.toml") {
            return home.to_path_buf();
        }

        if let Some(config_env) = std::env::var_os("GROK_HOME") {
            return PathBuf::from(config_env).join("config.toml");
        }

        let home_dir = if home.file_name().and_then(|n| n.to_str()) == Some(".grok") {
            home.parent().unwrap_or(home)
        } else {
            home
        };

        home_dir.join(".grok").join("config.toml")
    }
}
```

## 2. API Contract & Helper Functions

1. `pub fn register_grok_mcp_server(config_path: &Path, name: &str, command: &str, args: &[&str], env: &BTreeMap<String, String>) -> Result<(), CeError>`:
   - Parses `config_path` if present using `toml::Table`.
   - Ensures `mcp_servers` table exists, then inserts/updates `mcp_servers.<name>` with `command`, `args`, and optional `env` table.
   - Preserves all other TOML tables and keys.
   - Writes back atomically using `write_atomic`.

2. `pub fn unregister_grok_mcp_server(config_path: &Path, name: &str) -> Result<(), CeError>`:
   - Removes `name` from `mcp_servers` table.
   - Writes updated TOML back atomically. Leaves `config.toml` intact.

3. `pub fn update_grok_rule_md(rule_path: &Path, managed_text: &str) -> Result<(), CeError>`:
   - Updates `.grok/rules/compound-engineering.md` with demarcated `CE-AI MANAGED BLOCK`.

4. **Harness Probing & Detection**:
   - `HarnessKind::Grok.harness_dir(home_dir)`: Evaluates `$GROK_HOME` environment variable if set, defaulting to `home_dir.join(".grok")`.
   - `HarnessKind::Grok.is_installed_on_host`: Checks if `$GROK_HOME` exists or if `~/.grok` exists or if `config.toml` exists.
   - `HarnessKind::Grok.is_ce_installed`: Checks if `config.toml` contains `[mcp_servers]` sidecars or if `skills/` exists under the harness directory.

5. **Backup & Health Check Integration**:
   - `backups.rs`: Recognized with `grok-` prefix in backup storage.
   - `doctor.rs` & `status.rs`: Inspects `config.toml` `[mcp_servers]` for `codegraph` / `engram` status.
