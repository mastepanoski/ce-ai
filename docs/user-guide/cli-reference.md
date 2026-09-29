<!-- Diátaxis Quadrant: Reference | Audience: Developers -->
# CLI reference

This is the current command surface for `ce-ai`. Run `ce-ai <command> --help`
for complete arguments. Commands that mutate configuration support global
`--dry-run`; it previews their own writes, not external tool installation.

## Installation and recovery

| Command | Boundary |
| --- | --- |
| `install --harness <name>` | Installs managed CE assets into one harness; use `--harness all` for every detected harness. |
| `sync`, `upgrade`, `self-update` | Reconcile managed assets, update plugin source, or update the CLI binary. |
| `status`, `doctor`, `backups`, `uninstall` | Inspect state, diagnose health, browse/restore snapshots, or remove managed assets. |
| `init-prj`, `deinit-prj` | Add or remove the managed project workflow block. |

## Workflow and specifications

| Command | Boundary |
| --- | --- |
| `workflow`, `archive`, `graduate` | Persist/recover workflow state and manage ODD/OpenSpec change packages. |
| `gate check` | Enforces a complete Stage 4 OpenSpec contract by default; `--mode observe` records the decision without blocking. The Organic LOC notice is advisory. |
| `guard` | Persists and reports a pedagogical guardrail configuration marker. It does not independently enforce agent behavior. |
| `spec`, `doc` | Inspect/promote living specifications and inspect solution-library hygiene. |

## Harness configuration and analysis

| Command | Boundary |
| --- | --- |
| `models`, `skills`, `decisions` | Manage model settings, skill discovery, and decision-engine configuration/evaluation. |
| `tools status` | Reports companion readiness. |
| `tools install <tool>` | Registers an MCP definition; it does **not** download or install the third-party binary. |
| `tools init codegraph` | Initializes an already-installed CodeGraph CLI for a workspace. |
| `audit` | Reports local configuration-hygiene heuristics; it is not a token-usage measurement system. |
| `usage sync` | Captures local **Claude Code** transcript usage into the ledger. Other harness adapters are not implemented. |
| `usage report` | Renders ledger rows; `--from` and `--to` require inclusive RFC 3339 timestamps. `--by record` is the only supported grouping. |
| `report-bug` | Produces a sanitized internal ce-ai diagnostic report and asks for explicit submission consent. |
