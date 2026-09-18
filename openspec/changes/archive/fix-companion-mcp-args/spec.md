# Specification: Companion MCP Server Invocation Arguments

## Requirements

### Requirement 1: CodeGraph MCP Registration
- **WHEN** `ce-ai` registers `codegraph` as an MCP companion in any harness configuration (via `RegistrationSpec::register_companions`, `opencode::config::register_companions`, `harness::custom::register_companions`, or `ce-ai tools install codegraph`),
- **THEN** it MUST configure the command as `codegraph` and the argument list as `["serve", "--mcp"]`.
- **AND** it MUST NOT configure the argument list as `["mcp"]`.

### Requirement 2: Engram MCP Registration
- **WHEN** `ce-ai` registers `engram` as an MCP companion in any harness configuration (via `RegistrationSpec::register_companions`, `opencode::config::register_companions`, `harness::custom::register_companions`, or `ce-ai tools install engram`),
- **THEN** it MUST configure the command as `engram` and the argument list as `["mcp", "--tools=agent"]`.
- **AND** it MUST NOT configure the argument list as `["serve"]`.

### Requirement 3: Preservation of User Configurations
- **WHEN** companion MCP servers are registered or updated,
- **THEN** pre-existing user-defined MCP servers (such as `context7`, `linear`, `mercadopago`, `pencil`, `playwright`, etc.) MUST remain intact.

### Requirement 4: Verification Parity Across All Harnesses
- **WHEN** tests run across all supported native harnesses (`claude`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`, `custom`, `opencode`),
- **THEN** all registration and serialization tests MUST pass without error against the updated arguments.
