# Specification: Cursor Native Harness Adapter

## Requirements

### Requirement 1: Native MCP Server Schema
WHEN `ce-ai install --harness cursor` or `ce-ai tools install <tool> --harness cursor` is invoked,
THEN `ce-ai` SHALL write MCP server configurations into `~/.cursor/mcp.json` under the `mcpServers` root key using the `stdio` format.

### Requirement 2: User Configuration Preservation
WHEN `ce-ai` modifies `~/.cursor/mcp.json`,
THEN `ce-ai` SHALL preserve all unmanaged `mcpServers` entries and top-level JSON keys created by the user.

### Requirement 3: Zero OpenCode Key Leakage
WHEN `~/.cursor/mcp.json` is created or modified by `ce-ai`,
THEN `ce-ai` SHALL NOT write OpenCode-specific keys (`plugin`, `skills.paths`) into `~/.cursor/mcp.json`.

### Requirement 4: Native Rule File Format
WHEN `ce-ai init-prj --harness cursor` is invoked in a workspace,
THEN `ce-ai` SHALL write rule directives to `.cursor/rules/compound-engineering.mdc` with valid frontmatter (`description`, `globs`, `alwaysApply`) and demarcated managed comment blocks.

### Requirement 5: Clean Uninstall Lifecycle
WHEN `ce-ai uninstall --harness cursor` is executed,
THEN `ce-ai` SHALL restore pre-install backups or remove `ce-ai`-managed `mcpServers` entries and remove `~/.cursor/compound-engineering/`, restoring user files without leaving orphaned files.
