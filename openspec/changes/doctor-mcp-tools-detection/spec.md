# Specification: MCP-Configured Companion Tools & Skill Suggestions Detection

## Requirements

### Requirement 1: Companion Tool Detection via MCP Configuration
- **WHEN** a companion tool (such as `context7`) is registered under `mcpServers` in an active harness configuration file (e.g. `opencode.json`)
- **AND** no binary for the tool exists on `PATH`
- **THEN** `ce-ai doctor` SHALL report the companion tool as OK (`doctor-info: <name> v<version> (ok)`)
- **AND** `ce-ai doctor` (in either normal or `--strict` mode) SHALL NOT report a finding for `tool-missing`
- **AND** `ce-ai tools status` SHALL report the tool with status `✅` and hint `v<version> (ok)`.

### Requirement 2: Skill Suggestions Suppression for Configured Skills
- **WHEN** a skill suggestion entry (such as `sequential-thinking`) is registered under `mcpServers` in an active harness configuration OR is present in the skill registry
- **THEN** `ce-ai doctor` SHALL NOT output a `doctor-info: skill-suggestion: <name>` message for that skill
- **AND** `ce-ai tools status` SHALL NOT include that skill in the `[Skill Registry Suggestions]` list.

### Requirement 3: Missing Companion Tool and Skill Suggestion Preservation
- **WHEN** a companion tool is absent from both `PATH` and all candidate `mcpServers` configurations
- **THEN** `ce-ai doctor` SHALL report `companion tool '<name>' not found (suggested: '...')`
- **AND** `ce-ai doctor --strict` SHALL report a `tool-missing: ...` finding
- **AND** `ce-ai tools status` SHALL display `❌ <name> ... not found`.
- **WHEN** a skill suggestion entry is absent from all candidate `mcpServers` configurations and the skill registry
- **THEN** `ce-ai doctor` SHALL output `doctor-info: skill-suggestion: <name> (run '...')`
- **AND** `ce-ai tools status` SHALL display `⚠️ <name> ... (suggested: '...')`.
