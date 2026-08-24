# Design: Pi Native Harness Adapter

## System Architecture

### `PiAdapter` (Struct & `HarnessAdapter` Trait Implementation)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PiAdapter;

impl HarnessAdapter for PiAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Pi
    }

    fn default_config_path(&self, home_dir: &Path) -> PathBuf {
        self.kind().harness_dir(home_dir).join("skills")
    }

    fn canonical_instruction_file(&self) -> PathBuf {
        PathBuf::from("AGENTS.md")
    }

    fn derived_stub_files(&self) -> Vec<PathBuf> {
        vec![PathBuf::from(".pi").join("AGENTS.md")]
    }
}
```

## Environment Resolution & Directory Rules
1. If `$PI_CODING_AGENT_DIR` is set and non-empty, use that path as `harness_dir`.
2. Else default to `<home_dir>/.pi/agent`.
3. Skills directory is `harness_dir.join("skills")`.
4. Project rules target `AGENTS.md` (root) and `.pi/AGENTS.md` (when `.pi/` directory pre-exists).

## No-MCP Behavior
- `tools install` reports `pi` as unsupported for native MCP servers by design.
- No `config.json` or `mcp.json` file is created or mutated for `pi`.
