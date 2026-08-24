# Exploration: Pi Native Harness Adapter

## Technical Investigation
1. **Directory Location**:
   - Upstream default directory: `~/.pi/agent/`.
   - Override environment variable: `$PI_CODING_AGENT_DIR`.
   - `HarnessKind::Pi.harness_dir(home_dir)` evaluates `$PI_CODING_AGENT_DIR` if present and non-empty, falling back to `home_dir.join(".pi").join("agent")`.
2. **Skills Installation**:
   - `pi` natively loads skills from `~/.pi/agent/skills/<skill-name>/SKILL.md`.
   - `ce-ai install pi` copies managed compound engineering skills into `~/.pi/agent/skills/`.
3. **No MCP Server Config**:
   - `pi` does not maintain `mcp.json` or `config.json`.
   - `ce-ai tools install` for `pi` skips MCP config generation and outputs an informative message stating MCP is unsupported for `pi` by design.
