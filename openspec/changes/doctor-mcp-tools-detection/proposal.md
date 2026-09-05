# Proposal: MCP-Configured Companion Tools and Skills Detection in Doctor and Tools Status

## Problem Statement
`ce-ai doctor` and `ce-ai tools status` report companion tools like `context7` as `not found` (or `tool-missing` in `--strict` mode) and unconditionally print skill suggestions for `sequential-thinking`, even when both are already configured as MCP servers in the active harness configuration (`opencode.json`, `~/.cursor/mcp.json`, `claude_desktop_config.json`, etc.).

This false positive occurs because:
1. `extract_tool_version` only inspects `PATH` by executing `Command::new(tool_name).arg("--version")`. However, companion tools whose installation model is an MCP-server wrapper (such as `context7`, provisioned via `npx -y @upstash/context7-mcp@latest`) do not install a standalone binary on `PATH` by design.
2. The skill suggestions probe in `doctor.rs` and `tools.rs` iterates over all entries in `ToolsRegistryCache.skills` and outputs a recommendation without first checking whether the skill or MCP server is already configured or registered in the environment.

## Boundaries

### In Scope
- Implementing shared detection helpers in `src/source/tools_registry.rs`:
  - `is_mcp_server_configured(ctx: &Context, name: &str) -> bool`: Checks active harness configurations (`opencode.json` in global or resolved workspace directory, and native harness MCP config files) for `mcpServers` entries matching the target name or invocation command/args.
  - `is_skill_configured(ctx: &Context, name: &str) -> bool`: Checks whether a skill suggestion is configured either as an MCP server or in `skills-registry.json`.
  - `detect_tool_freshness(ctx: &Context, name: &str, info: &CompanionToolInfo) -> FreshnessStatus`: Combines CLI `--version` probing on `PATH` with MCP configuration fallback.
- Updating `ce-ai doctor` (`src/commands/doctor.rs`) to use `detect_tool_freshness` and `is_skill_configured`.
- Updating `ce-ai tools status` (`src/commands/tools.rs`) to use `detect_tool_freshness` and filter out already-configured skill suggestions.
- Adding regression tests in `tests/cli.rs` and unit tests covering MCP detection.

### Out of Scope
- Changing how `ce-ai tools install` provisions MCP servers.
- Adding new companion tools or skills to the registry.

## Risk Evaluation
- **False Negative Risk**: If a tool is genuinely missing from both PATH and MCP server configurations, it must still be reported as `not found` (or `tool-missing` with `--strict`), and skill suggestions must still be emitted.
- **Config Corruption Risk**: Probing harness configurations is read-only; no modifications to harness config files are performed during doctor or status commands.

## Success Criteria
- WHEN `context7` is configured in `mcpServers` in the active harness config, `ce-ai doctor` and `ce-ai tools status` report `context7` as OK (`v1.0.0 (ok)`), not `not found`, even when no `context7` binary exists on `PATH`.
- WHEN `sequential-thinking` is configured in `mcpServers` in the active harness config, `ce-ai doctor` does not print `doctor-info: skill-suggestion: sequential-thinking ...`.
- WHEN neither is configured, the existing `not found` and `skill-suggestion` outputs are preserved.
- 100% test pass rate across `cargo test`.
