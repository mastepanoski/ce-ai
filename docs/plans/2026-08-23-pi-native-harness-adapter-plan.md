# Implementation Plan: Pi Native Harness Adapter

Implement native `PiAdapter` targeting Mario Zechner's `pi` coding agent (`~/.pi/agent/`, `$PI_CODING_AGENT_DIR`, skills under `~/.pi/agent/skills/`, zero MCP config).

## User Review Required
> [!IMPORTANT]
> `pi` explicitly operates without native MCP configuration by design ("No MCP philosophy"). `ce-ai` will manage skills under `~/.pi/agent/skills/` and instructions via `AGENTS.md`, and report MCP as unsupported for `pi` targets in `tools install`.

## Proposed Changes

### 1. Harness Adapter (`src/harness/pi.rs` & `src/harness/mod.rs`)
- Update `harness_dir` for `HarnessKind::Pi`:
  - Check `$PI_CODING_AGENT_DIR` if set and non-empty.
  - Fall back to `home_dir.join(".pi").join("agent")`.
- Implement `PiAdapter` in `src/harness/pi.rs`:
  - `kind()` -> `HarnessKind::Pi`.
  - `default_config_path(home_dir)` -> `harness_dir(home_dir).join("skills")`.
  - `canonical_instruction_file()` -> `AGENTS.md`.
  - `derived_stub_files()` -> `vec![.pi/AGENTS.md]`.
- Update `HarnessKind::is_installed_on_host` and `is_ce_installed` in `src/harness/mod.rs`.

### 2. Command Integration
- `src/commands/install.rs`:
  - Handle `HarnessKind::Pi`: copy managed skills to `config_dir.join("skills")` (`~/.pi/agent/skills/`), write manifest without fictional MCP JSON or plugin keys.
- `src/commands/uninstall.rs`:
  - Clean up `skills_dir` (`~/.pi/agent/skills/`) for `HarnessKind::Pi`.
- `src/commands/tools.rs`:
  - If `tools install` targets `pi`, skip writing MCP config and report `"info: pi harness does not support native MCP servers by design"`.
- `src/commands/init_prj.rs` & `deinit_prj.rs`:
  - Adopt `.pi/AGENTS.md` when `.pi/` directory exists in the project root.

### 3. OpenSpec & Documentation
- Create OpenSpec contract under `openspec/changes/pi-native-harness-adapter/` (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- Document solution in `docs/solutions/architecture/pi-native-harness-adapter.md`.

## Verification Plan
- Unit tests in `src/harness/pi.rs`.
- Integration tests in `tests/cli.rs`: `install_pi_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `uninstall_pi_harness_clean_lifecycle`.
- Quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`).
