# Exploration: Companion MCP Registration Directory Guard

## Architectural Investigation

### Current Behavior & Call Flow
During `ce-ai install` (`src/commands/install.rs`) and `ce-ai sync` (`src/commands/sync.rs`), candidate harnesses are iterated. For each table-driven harness:

1. `let spec = registration_spec(*harness_kind)` fetches the registration strategy.
2. `spec.register_companions(&target_config)?` is called.
3. In `src/harness/registration.rs:29`:
   ```rust
   pub(crate) fn register_companions(&self, target_config: &Path) -> Result<(), CeError> {
       let Some(register) = self.register_mcp else {
           return Ok(());
       };
       let env = BTreeMap::new();
       register(target_config, "codegraph", "codegraph", &["mcp"], &env)?;
       register(target_config, "engram", "engram", &["serve"], &env)?;
       Ok(())
   }
   ```
4. `register` is a function pointer `McpRegistrar` pointing to native registrars (e.g. `register_claude_mcp_server`, `register_cursor_mcp_server`, etc.).
5. Each registrar checks `if config_path.exists()` and immediately invokes `std::fs::read_to_string(config_path)`.
6. When `config_path` is a directory (such as when a directory was accidentally created or mapped at that path), `std::fs::read_to_string` fails with `EISDIR (os error 21)`.
7. `CeError::Io` has no path information, formatting as `error: I/O error: Is a directory (os error 21)`.
8. The error aborts the `install` loop. `state.json` is never written, and the mutations ledger is left uncompleted, causing full rollback on the next invocation.

### Evaluated Options

#### Option A: Duplicate Guards in `install.rs` and `sync.rs`
- Add `if target_config.is_file()` checks before calling `spec.register_companions(&target_config)`.
- **Drawbacks**: Violates DRY; leaves `spec.register_companions` fragile if called from other contexts or future commands; doesn't provide wrapped I/O error context inside registration.

#### Option B: Patch All 9 Native Adapters
- Modify `claude.rs`, `cursor.rs`, `codex.rs`, `copilot.rs`, `grok.rs`, `kimi.rs`, `agy.rs`, `fx.rs`, and `custom.rs` to check `if config_path.is_file()`.
- **Drawbacks**: High blast radius (9 files modified), code duplication across adapters, risk of subtle regressions or missed adapters.

#### Option C: Centralized Guard in `RegistrationSpec::register_companions` (Chosen)
- Add `kind: HarnessKind` to `RegistrationSpec`.
- In `RegistrationSpec::register_companions`:
  - Check `if target_config.exists() && !target_config.is_file()`.
  - Emit an explicit warning to stderr (`eprintln!`).
  - Return `Ok(())` to allow continuation.
  - Wrap any unexpected `CeError::Io` from `register(...)` with harness and path context.
- **Benefits**: Single point of control, zero code duplication, zero changes needed to the 9 native adapters, fully backward-compatible with existing callers.

## Decisions & Tradeoffs
- **Warning over silent no-op**: Per project governance rules (no dummy fallbacks or silent error swallowing), skipping must be explicitly reported via a warning banner:
  `warn: skipping companion MCP registration for <harness>: '<path>' exists but is not a regular config file (expected a JSON/TOML file)`
- **Error wrapping**: If `register(...)` returns `CeError::Io(err)`, we wrap it in a new `std::io::Error` containing `format!("{}: '{}': {err}", self.kind, target_config.display())` while preserving the I/O error category and exit code 4.
