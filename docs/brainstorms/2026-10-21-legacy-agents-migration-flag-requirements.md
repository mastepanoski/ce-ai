# Feature Requirements: Legacy AGENTS.md Assisted Migration (`ce-ai init-prj --migrate`)

**Date**: 2026-10-21  
**Status**: Draft / Brainstorm Concluded  
**Target Module**: `src/commands/init_prj.rs`, `src/commands/doctor.rs`  
**Related Documents**: `docs/ideation/2026-10-21-legacy-agents-fsm-migration-ideation.html`

---

## 1. Context & Problem Statement

Between August and late September 2026, early versions of `ce-ai` enforced a rigid, blocking Finite State Machine (FSM) and mandatory OpenSpec authoring for all changes:
- **Legacy Invariant 6**: *"OpenSpec Required: NO code changes without formal spec in `openspec/changes/<feature_name>/`"*.
- **Legacy Stage 2**: *"Stage 2: OpenSpec Definition (MANDATORY)"*.
- **Rigid FSM**: Workflows expected 100% stage completion before any PR, blocking small bug fixes and routine edits behind formal 5-artifact specifications (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).

Starting with `ce-ai` v1.77.0 and consolidated in v2.0/v3.0:
1. OpenSpec was decoupled into an optional architectural tool, establishing the **No Semantic Authority** rule.
2. The FSM was transformed into an **Advisory Observation Engine** (`GateMode::Observe`).
3. A proactive triage heuristic was introduced (suggest OpenSpec for architectural changes / > 200 LOC; bypass for bug fixes / < 200 LOC).

### The "Cognitive Collision" Pathology
In consumer projects that adopted `ce-ai` during earlier phases or manually authored `AGENTS.md` before managed comment blocks (`<!-- ce-ai:block begin ... -->`):
- Re-running `ce-ai init-prj` simply appends the modern managed block to the bottom of `AGENTS.md`.
- As a result, `AGENTS.md` contains conflicting instructions:
  - Top (unmanaged): Rigid negative constraints (*"NO code changes without formal spec"*, *"Stage 2 MANDATORY"*).
  - Bottom (managed): Advisory guidelines and optional OpenSpec.
- Modern AI models (Claude, Gemini, Cursor, Copilot) read the file top-down and prioritize strict negative constraints. Agents freeze or refuse to make 2-line fixes without authoring 5 OpenSpec documents, paralyzing developer workflow.

---

## 2. Goals & Non-Goals

### Goals
- **Surgical Non-Destructive Modernization**: Provide an assisted migration flag `ce-ai init-prj --migrate [path]` that identifies and rewrites known legacy CE-AI directives in `AGENTS.md` without modifying any host project domain rules (linters, test commands, architecture notes).
- **Safety First & Atomic Backups**: Automatically create `AGENTS.md.bak` before making mutations, and support `--dry-run` to output unified diffs for verification before disk writes.
- **Fleet-Wide Batch Migration**: Support `--all` (`ce-ai init-prj --migrate --all`) to scan and upgrade all adopted repositories registered in `state.json`.
- **Companion Harness Reconciliation**: Ensure companion rule files (`CLAUDE.md`, `.cursor/rules`, `.github/copilot-instructions.md`, `.grok/rules/`) are reconciled to prevent cross-tool drift.
- **Diagnostic Integration**: Update `ce-ai doctor` to detect unmanaged legacy patterns and recommend running `ce-ai init-prj --migrate`.

### Non-Goals
- **Total `AGENTS.md` Overwrites**: The tool must never overwrite unmanaged content blindly or wipe custom project sections.
- **Universal Markdown Refactoring**: The tool will not format or rewrite arbitrary markdown syntax outside of recognized CE-AI historical clauses.

---

## 3. User Experience & CLI Specification

### 3.1 Single Project Migration
```bash
# Preview changes without modifying disk
ce-ai init-prj --migrate ./my-project --dry-run

# Apply migration with automatic backup (AGENTS.md.bak)
ce-ai init-prj --migrate ./my-project
```

### 3.2 Fleet-Wide Batch Migration
```bash
# Preview migration across all registered projects
ce-ai init-prj --migrate --all --dry-run

# Execute batch migration across the registered fleet
ce-ai init-prj --migrate --all
```

### 3.3 Terminal Output & Visual Feedback
When `--dry-run` is supplied, `ce-ai` outputs:
```text
🔍 Scanning ./my-project/AGENTS.md for legacy directives...
Found 3 legacy patterns:
  - [L12] Legacy Invariant 6 (Strict OpenSpec Enforcement)
  - [L85] Mandatory Stage 2 FSM Diagram
  - [L142] Deprecated CLI command reference ('ce-ai models')

Proposed Diff:
--- AGENTS.md (legacy)
+++ AGENTS.md (migrated)
@@ -12,2 +12,2 @@
-6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.
+6. **No Semantic Authority & Optional OpenSpec**: CE-AI MUST NOT introduce mandatory workflow stages...
...
Run without --dry-run to apply this migration. A backup will be saved to AGENTS.md.bak.
```

---

## 4. Transformation Engine & Pattern Rules

The migration engine targets specific, deterministic clauses:

| Pattern ID | Legacy Pattern in `AGENTS.md` | Modern Replacement |
|---|---|---|
| `PAT-INV-6` | `OpenSpec Required: NO code changes without formal spec...` | `No Semantic Authority & Optional OpenSpec: CE-AI MUST NOT introduce mandatory workflow stages, artifacts, or blocking write gates...` |
| `PAT-FSM-7` | `[Stage 2: OpenSpec Definition (MANDATORY)]` | `[Stage 2: OpenSpec Definition (OPTIONAL)]` |
| `PAT-STAGE-2` | `### Stage 2 OpenSpec Enforcement Requirements\n...contains all five files:` | `### Stage 2 OpenSpec Integration (Optional)\n...contains:` + Proactive OpenSpec Triage & Guidance (200 LOC ceiling) |
| `PAT-CLI-MODELS` | `ce-ai models set/list/profile` | `ce-ai fleet` / `ce-ai guard` |
| `PAT-CLI-DECISIONS` | `ce-ai decisions route model advisor` | `ce-ai guard agent guardrails & risk policy engine` |
| `PAT-FSM-READY` | `so the repository FSM is left at ✓ Ready (100%)` | `and archive completed OpenSpec changes via ce-ai archive <feature>` |

### Preservation Guarantee
Any block not matching a recognized CE-AI historical clause (such as custom database setups, API design guidelines, domain invariants 1-5, or code style directives) MUST be preserved character-for-character.

---

## 5. Multi-Harness & Fleet Reconciliation

When `--migrate` runs:
1. Mutates `AGENTS.md` atomically using `crate::state::write_atomic`.
2. Updates or injects the modern demarcated block (`BLOCK_VERSION = 7` or higher) with the designated tier (`full`, `minimal`, or `orchestrator`).
3. Reconciles companion harness files:
   - `CLAUDE.md`: verified or generated as `@AGENTS.md`.
   - `.cursor/rules/compound-engineering.mdc`: updated with the managed rule block.
   - `.github/copilot-instructions.md`: updated with the managed block.
   - `.grok/rules/compound-engineering.md`: updated with the managed block.
   - RTK hooks: reconciled if supported.

---

## 6. Diagnostic Integration (`ce-ai doctor`)

`ce-ai doctor` will add a check `check_unmanaged_legacy_directives`:
- If `AGENTS.md` contains unmanaged legacy patterns like `PAT-INV-6` or `PAT-FSM-7`:
  - Outputs warning: `⚠️  project-adoption: legacy unmanaged FSM / mandatory OpenSpec rules detected in 'AGENTS.md' outside managed block`.
  - Recommends: `Run 'ce-ai init-prj --migrate <path>' to safely modernize directives without losing domain rules.`
- If `ce-ai doctor --fix` is run, it can automatically invoke the migration engine if `--fix` is active.

---

## 7. Verification & Acceptance Criteria

1. **Unit Tests (`src/commands/init_prj.rs`)**:
   - `test_migrate_legacy_agents_md_replaces_invariant_6_and_preserves_custom_notes`: verifies exact string preservation of custom domain rules while modernizing Invariant 6 and Stage 2.
   - `test_migrate_dry_run_does_not_modify_disk`: confirms disk is untouched when `--dry-run` is active.
   - `test_migrate_creates_backup_file`: verifies `AGENTS.md.bak` matches the pre-migration content.
2. **Integration Tests (`tests/cli.rs`)**:
   - `init_prj_migrate_legacy_fixture_upgrades_cleanly`: runs `ce-ai init-prj --migrate <dir>` against a real test repository fixture containing early v1.0 `AGENTS.md` content.
   - `doctor_detects_legacy_unmanaged_directives_and_reports_migration_hint`: verifies CLI exit codes and output messages.
3. **Quality Gates**:
   - Passes `cargo fmt --check`.
   - Passes `cargo clippy --all-targets --all-features -- -D warnings`.
   - Passes `cargo test`.
