# Specification: Skills Resolve Positional Query Support

## Requirements

### REQ-1: Positional Query Invocation
- **WHEN** the user invokes `ce-ai skills resolve <QUERY>` (e.g. `ce-ai skills resolve sequential-thinking`),
- **THEN** `ce-ai` SHALL accept the argument without a Clap parsing error, resolve `<QUERY>` against the skills and companion tools registries, and output the resolved prompt / configuration instructions.

### REQ-2: Backward-Compatible Flag Invocation
- **WHEN** the user invokes `ce-ai skills resolve --query <QUERY>`,
- **THEN** `ce-ai` SHALL accept the flag as before and execute the query resolution identically to REQ-1.

### REQ-3: JSON Flag Support with Positional Argument
- **WHEN** the user invokes `ce-ai skills resolve <QUERY> --json`,
- **THEN** `ce-ai` SHALL emit the structured JSON payload containing the resolved candidate skill, installation commands, and prompt.

### REQ-4: Missing Query Handling
- **WHEN** the user invokes `ce-ai skills resolve` with neither a positional query nor a `--query` flag,
- **THEN** `ce-ai` SHALL return an error with exit code 2 (`CeError::Usage`), explaining that a query must be provided.

## Acceptance Criteria
1. `ce-ai skills resolve sequential-thinking` exits with code 0 and prints markdown containing `sequential-thinking`.
2. `ce-ai skills resolve --query sequential-thinking` exits with code 0 and prints identical markdown.
3. `ce-ai skills resolve sequential-thinking --json` exits with code 0 and prints valid JSON.
4. `ce-ai skills resolve` (with no arguments) exits with code 2.
5. All CLI tests pass without regression.
