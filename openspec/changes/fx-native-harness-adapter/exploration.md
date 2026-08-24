# Exploration: fx Native Harness Adapter

## Technical Investigation
1. **Directory Resolution**:
   - Default directory: `~/.fx`.
   - Override variable: `$FX_HOME`.
   - `HarnessKind::Fx.harness_dir(home_dir)` evaluates `$FX_HOME` if set and non-empty, falling back to `home_dir.join(".fx")`.
2. **MCP JSON Format**:
   - File location: `<harness_dir>/mcp.json`.
   - Root key: `mcp` (`BTreeMap<String, FxMcpServer>`).
   - Server structure:
     - `r#type`: `Option<String>` (`"local"` for stdio command servers).
     - `command`: `Vec<String>` (array-form command, where first element is binary and remaining elements are arguments).
     - `environment`: `BTreeMap<String, String>` map.
     - `extra`: `serde_json::Map<String, Value>` to preserve extra fields (e.g. `enabled`, `required`).
3. **Skills Location**:
   - `<harness_dir>/skills/<skill-name>/SKILL.md`.
4. **Project Rules Location**:
   - Target `AGENTS.md` at project root by default, and additionally target `.fx/AGENTS.md` when `.fx/` directory exists.
