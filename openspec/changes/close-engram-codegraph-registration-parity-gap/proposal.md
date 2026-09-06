# Proposal: Close engram/codegraph registration parity gap for Opencode, Custom, Deepseek, Pi

## 1. Problem Statement
Companion MCP servers (`codegraph` and `engram`) are auto-registered during `ce-ai install` and `ce-ai sync` for native harnesses managed via the strategy table `registration_spec` in `src/harness/registration.rs` (Claude, Codex, Copilot, Grok, Kimi, Agy, Fx, Cursor). However, three harnesses currently return `None`:
```rust
HarnessKind::Custom | HarnessKind::Opencode | HarnessKind::Deepseek => return None,
```
Additionally, Pi is configured with `RegistrationSpec { register_mcp: None }` by architectural design ("No-MCP by design: skills tree only"), which currently lacks a distinct, documented delivery model.

As a result:
1. **Opencode**: Does not auto-register `codegraph` and `engram` during `install` or `sync`, forcing users to run `ce-ai tools install engram` manually.
2. **Custom**: Has no auto-registration for companion tools, even though users may specify custom configurations with an MCP configuration file.
3. **Deepseek**: Is de-scoped, but this decision is undocumented in the registration table and appears as a silent omission.
4. **Pi**: Is No-MCP by design, but lacks an explicit non-MCP delivery contract and doctor reporting mechanism.
5. **RegistrationSpec / registration_spec**: Completely lacks dedicated characterization unit tests.

## 2. In-Scope / Out-of-Scope Boundaries
- **In-Scope**:
  - Add characterization test suite for `RegistrationSpec` and `registration_spec` in `src/harness/tests/registration.rs`.
  - Add auto-registration of `codegraph` and `engram` into `opencode.json` during `ce-ai install` and `ce-ai sync` for OpenCode using its existing `crate::opencode::config::register_mcp_server`.
  - Extend `CustomHarnessConfig` to support an optional `mcp_file: Option<PathBuf>` (via `--mcp-file` CLI flag and `custom_harness.json`), and register companions into it during `install` and `sync` if configured.
  - Formulate and document the architectural decision for Deepseek remaining de-scoped (YAML patch layers under `~/.dsh`, developer preview).
  - Define and document the delivery contract for Pi (CLI binaries on PATH + skills tree, No-MCP by design) and ensure `doctor` reports Pi companion readiness accurately.
  - Update `find_mcp_config_paths` in `src/source/tools_registry.rs` to include `opencode.json` and Custom's `mcp_file`.
  - Bump SemVer to `1.41.0` (MINOR for companion parity feature) and update `CHANGELOG.md`.
- **Out-of-Scope**:
  - `rtk` (hook-based command rewriting, tracked in Issue #308).
  - `sequential-thinking` (MCP vs on-demand skills resolution, tracked in Issue #309).
  - Adding MCP capabilities to Pi (violates objective 8 No-MCP invariant).
  - Implementing full DeepSeek `dsh` YAML layer engine.

## 3. Risk Evaluation
- **Token Neutrality & Non-Interference**: Writing `mcpServers` entries into `opencode.json` or custom MCP files preserves existing user servers using `register_mcp_server`'s non-destructive merging.
- **Pi Stability**: Pi remains strictly No-MCP; no foreign MCP structures are injected into its directories.

## 4. Success Criteria
- Characterization unit tests cover all `HarnessKind` variants in `registration_spec`.
- `ce-ai install --harness opencode` and `ce-ai sync` write `codegraph` and `engram` into `opencode.json`'s `mcpServers`.
- `ce-ai install --harness custom --mcp-file <PATH>` registers companions into the target file.
- `ce-ai doctor` detects configured companions across OpenCode and Custom, and recognizes Pi's CLI-based skills delivery.
