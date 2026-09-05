# Spec: Close engram/codegraph registration parity gap

## Requirements

### Requirement 1: OpenCode Companion Auto-Registration
WHEN `ce-ai install` or `ce-ai sync` runs for `HarnessKind::Opencode`
THEN `opencode.json` SHALL contain MCP server entries for `codegraph` (`command: "codegraph", args: ["mcp"]`) and `engram` (`command: "engram", args: ["serve"]`) under `mcpServers`.

### Requirement 2: Custom Harness Companion Registration
WHEN `ce-ai install` or `ce-ai sync` runs for `HarnessKind::Custom`
AND `CustomHarnessConfig.mcp_file` is specified (via `--mcp-file` or `custom_harness.json`)
THEN the target `mcp_file` SHALL contain MCP server definitions for `codegraph` and `engram` under `mcpServers`.
WHEN `CustomHarnessConfig.mcp_file` is absent
THEN companion MCP registration SHALL be skipped cleanly without error.

### Requirement 3: Explicit Rationale for Deepseek De-scoping
`registration_spec(HarnessKind::Deepseek)` SHALL return `None`, accompanied by explicit code documentation explaining that DeepSeek CLI (`dsh`) uses YAML patch layers under `~/.dsh` and is in developer preview.

### Requirement 4: Pi No-MCP Delivery Model
`registration_spec(HarnessKind::Pi)` SHALL return `Some(RegistrationSpec { register_mcp: None })`, preserving the No-MCP by design invariant. Pi companion integration SHALL be defined as CLI execution via PATH binaries and the skills tree.

### Requirement 5: Exhaustive Registration Characterization Tests
All variants of `HarnessKind` and methods of `RegistrationSpec` SHALL be verified by dedicated unit tests in `src/harness/tests/registration.rs`.
