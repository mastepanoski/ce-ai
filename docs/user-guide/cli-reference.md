<!-- Diátaxis Quadrant: Reference | Audience: Developers -->
# CLI reference

This is the current command surface for `ce-ai`. Run `ce-ai <command> --help`
for complete arguments. Commands that mutate configuration support global
`--dry-run`; it previews their own writes, not external tool installation.

## Fleet Version Governance

| Command | Boundary |
| --- | --- |
| `fleet pin <version\|latest>` | Atomically records the target Compound Engineering semantic version in `state.json` (or dynamically resolves `latest` upstream release). |
| `fleet status [--json] [--system <id>]` | Audits and reports installed coding agent harnesses, version parity, and fleet alignment status (`aligned`, `divergent`, `missing`). Optionally filters by constellation system identity declared in `docs/arch/system.yaml`. |
| `fleet sync [--dry-run]` | Drives native host package managers and configuration files across all detected harnesses (`opencode`, `claude`, `pi`, `codex`, `cursor`) to align with the pinned version. |

## Project Adoption and Environment Health

| Command | Boundary |
| --- | --- |
| `init-prj [--tier <tier>] [--all] [--migrate] [--migrate-arch]` | Injects non-destructive `AGENTS.md` rules, session-start hooks, and RTK token compression into the current repository. Use `--migrate` to surgically modernize historical unmanaged CE-AI clauses into decoupled Compound Engineering guidance (with automatic `AGENTS.md.bak` backup). Use `--migrate-arch` to migrate legacy `openspec/` to living architecture in `docs/arch/`. Use `--all` to batch-upgrade or batch-migrate all registered projects. |
| `deinit-prj` | Safely removes managed blocks and session hooks from the current repository without touching user code or custom rules. |
| `doctor [--fix] [--migrate-arch] [--all-projects]` | Diagnoses environment readiness, harness health, companion tools, project adoptions, and `system.yaml` manifest integrity. Use `--fix` for automated repair of drifted blocks, missing hooks, and stale adoptions. Use `--migrate-arch` for assisted architecture migration. |
| `status`, `backups`, `uninstall` | Inspect state, browse/restore pre-mutation configuration snapshots, or remove managed assets. |
| `install`, `sync`, `upgrade` | *Legacy surfaces*: Emit deprecation notices guiding users to `init-prj` and `fleet pin / sync`. |

## Workflow and Specifications

| Command | Boundary |
| --- | --- |
| `workflow`, `archive` | Persist/recover workflow state, read SSOT progress from `docs/plans/`, and archive completed features into `docs/arch/archive/` (or legacy `openspec/changes/archive/`). |
| `graduate` (deprecated) | Legacy command for migrating historical `odd/tasks/` briefs to OpenSpec; new work uses `docs/plans/` or direct OpenSpec changes. |
| `gate check` | Enforces a complete Stage 4 contract by default; `--mode observe` records the decision without blocking. The agile LOC notice is advisory. |
| `spec`, `doc` | Inspect/promote living specifications (`docs/arch/system/` or `openspec/specs/`) and run documentation hygiene/linting (`doc lint`, `doc refresh`, `doc stats`). |

## Agent Guardrails & Risk Execution

| Command | Boundary |
| --- | --- |
| `guard check <tool> <cmd>` | Evaluates execution risk and safety policy for commands or tool invocations using deterministic precedence (<1 ms) and local semantic risk scoring. |
| `guard status`, `setup`, `test` | Inspects operational guardrails, applies local-first presets, and executes self-test verification probes. |
| `guard enable`, `disable` | Toggles pedagogical guardrail mode and level (`junior`, `strict`) across developer harnesses. |

## Decisions, Companions and Telemetry

| Command | Boundary |
| --- | --- |
| `decisions check-risk` | Backward-compatible alias forwarding directly to `ce-ai guard check`. |
| `decisions route` (deprecated) | Deprecated advisory command; modern AI coding harnesses manage model selection natively via CLI flags or configuration files. |
| `skills list`, `skills resolve` | Inspects and resolves available Compound Engineering skills across installed harnesses. |
| `tools status` | Reports companion readiness and health: **CodeGraph** (AST indexer), **Engram** (persistent memory), and **RTK** (token reducer). |
| `tools init codegraph` | Initializes the `.codegraph/` index in the current workspace. |
| `tools install <tool>` | Registers an MCP definition; does **not** download or install third-party binaries. |
| `usage sync [--harness <name>]` | Captures local transcript usage into the ledger from supported harnesses (`claude`, `opencode`, `codex`, `pi`). |
| `usage report` | Renders usage ledger rows; `--from` and `--to` require inclusive RFC 3339 timestamps. |
| `report-bug` | Produces a sanitized internal `ce-ai` diagnostic report for CLI errors and crashes, requiring explicit user consent. |
