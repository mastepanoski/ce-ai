# Specification: CLI audit alignment

## Requirements

### R1. Effective usage date filters
WHEN `ce-ai usage report` receives valid `--from` and/or `--to` timestamps
THEN it MUST output only records within the inclusive interval.

### R2. Honest usage argument handling
WHEN `ce-ai usage report` receives an invalid timestamp, an inverted interval,
or unsupported `--by` value
THEN it MUST return `CeError::Usage` with exit code 2 and MUST NOT silently
ignore the input.

### R3. Guard scope safety
WHEN `ce-ai guard disable --harness <name>` names a harness different from the
configured guard scope
THEN it MUST fail with a usage error and preserve the existing guard state.

### R4. Truthful CLI descriptions
WHEN users run `ce-ai --help` or relevant subcommand help
THEN descriptions MUST accurately identify Claude-only usage capture,
enforcing-by-default gate behavior, configuration-only guard behavior,
MCP-registration-only tools install behavior, and heuristic audit scope.

### R5. Accurate public documentation
WHEN a user follows README or user-guide CLI examples
THEN every example MUST use valid current argument syntax, and a linked CLI
reference MUST make every top-level command discoverable.
