# Proposal: Skills Resolve Positional Query Support

## 1. Problem Statement
When running `ce-ai doctor` or `ce-ai tools status`, `ce-ai` recommends installation/resolution commands for companion tools such as `sequential-thinking`. The suggested command is:
```bash
ce-ai skills resolve sequential-thinking
```
However, running this exact command fails with a Clap argument parsing error (exit code 2):
```
error: unexpected argument 'sequential-thinking' found

  Usage: ce-ai skills resolve [OPTIONS]

For more information, try '--help'.
```
This failure occurs because `Action::Resolve` in `src/commands/skills.rs` currently defines only named options:
- `--harness <HARNESS>` (default "opencode")
- `--query <QUERY>`
- `--json`

No positional argument is declared in the `Action::Resolve` clap enum variant. Consequently, users following the diagnostic hints in `ce-ai doctor` and `ce-ai tools status` encounter an immediate CLI failure.

## 2. In-Scope / Out-of-Scope Boundaries
- **In-Scope**:
  - Update `Action::Resolve` in `src/commands/skills.rs` to accept an optional positional query argument (`[QUERY]`) while retaining the named `--query` flag.
  - Prioritize positional query if present, falling back to `--query`, or defaulting to empty string.
  - Maintain 100% backward compatibility for existing scripts, invocations, and tests using `--query`.
  - Add comprehensive CLI integration tests in `tests/cli.rs` covering positional queries, named flag queries, and combined fallback behavior.
  - Bump SemVer to `1.39.2` in `Cargo.toml` and update `CHANGELOG.md`.
- **Out-of-Scope**:
  - Modifying the companion registry schema or network resolution logic in `src/source/tools_registry.rs`.
  - Changing other `ce-ai skills` subcommands (`list`, etc.).

## 3. Risk Evaluation
- **Argument Ambiguity**: By defining `query_pos: Option<String>` with `#[arg(value_name = "QUERY")]` alongside `query: Option<String>` with `#[arg(long)]`, clap cleanly distinguishes positional arguments from option flags.
- **Backward Compatibility**: Any scripts calling `ce-ai skills resolve --query <val>` continue to work identically.

## 4. Success Criteria
- `ce-ai skills resolve sequential-thinking` succeeds without clap errors and outputs the resolved companion skill prompt.
- `ce-ai skills resolve --query sequential-thinking` continues to function identically.
- `ce-ai skills resolve sequential-thinking --json` outputs valid JSON with the resolved skill.
- All unit, integration, and security tests pass green.
