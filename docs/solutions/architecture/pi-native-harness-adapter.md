---
module: harness
tags: [pi, harness, adapter, skills, agend_md, no_mcp]
problem_type: architectural_feature
---

# Solution: Pi Native Harness Adapter

## Problem
`ce-ai`'s previous stub for Mario Zechner's `pi` coding agent assumed a fictional `~/.pi/config.json` file with OpenCode-style schema.
According to official `pi` documentation (`pi.dev` / `@earendil-works/pi-coding-agent`):
1. Managed assets live under `~/.pi/agent/` (environment variable override `$PI_CODING_AGENT_DIR`).
2. `pi` intentionally has **no native MCP server configuration** by design ("No MCP philosophy").
3. `pi` loads instructions from `AGENTS.md` (root) and `.pi/AGENTS.md`.
4. `pi` loads skills from `~/.pi/agent/skills/`.

## Solution Details
1. **`PiAdapter` Implementation** (`src/harness/pi.rs`):
   - `kind()` -> `HarnessKind::Pi`.
   - `harness_dir(home_dir)` -> checks `$PI_CODING_AGENT_DIR` if set and non-empty, falling back to `home_dir.join(".pi").join("agent")`.
   - `default_config_path(home_dir)` -> `harness_dir(home_dir).join("skills")`.
   - `canonical_instruction_file()` -> `PathBuf::from("AGENTS.md")`.
   - `derived_stub_files()` -> `vec![PathBuf::from(".pi").join("AGENTS.md")]`.
2. **Skills Asset Installation** (`src/commands/install.rs`):
   - Copies managed skills to `~/.pi/agent/skills/` without fabricating fictional MCP JSON or plugin config files.
3. **No-MCP Behavior** (`src/commands/tools.rs`):
   - `ce-ai tools install` gracefully reports `pi` as unsupported for native MCP servers by design without failing multi-harness installations.
4. **Project Rule Adoption** (`src/commands/init_prj.rs` & `deinit_prj.rs`):
   - Adopts `.pi/AGENTS.md` when `.pi/` directory pre-exists in the project root.
5. **Uninstall Lifecycle** (`src/commands/uninstall.rs`):
   - Cleans up `~/.pi/agent/skills/` while preserving custom user files.
   - Updated `uninstall.rs` to check `target_config.is_file()` before calling `remove_file`, preventing directory removal errors when `target_config` is a directory.

## Verification
- Unit tests in `src/harness/pi.rs` testing path resolution and `$PI_CODING_AGENT_DIR`.
- Integration tests in `tests/cli.rs` (`install_pi_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `uninstall_pi_harness_cleans_native_dir_artifacts_and_preserves_user_configs`, `install_pi_harness_respects_pi_coding_agent_dir_env`).
- 100% green test suite (138 unit tests, 76 integration tests).
