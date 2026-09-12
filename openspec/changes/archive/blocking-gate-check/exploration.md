# Technical Exploration: Blocking Gate Check & Validation Receipt (Issue #334)

## 1. Technical Context & Investigation

### 1.1 Claude Code `PreToolUse` Hook Protocol
Claude Code executes `PreToolUse` hooks registered in `.claude/settings.json` whenever an agent invokes targeted tools (`Write|Edit`).
Through empirical analysis of the Claude Code runtime binary (`/Users/mastepanoski/.local/share/claude/versions/2.1.268`), the exact exit code and I/O contract was confirmed:

```text
PreToolUse Hook Behavior:
- Exit code 0: Hook passes; tool call proceeds normally. Stdout/stderr not shown.
- Exit code 2: Tool execution BLOCKED; stderr fed directly into model context.
- Other exit codes (1, 3, 4, 5, 6): Stderr shown to user only; Claude Code continues with tool call!
```

**Crucial Implication:**
In `ce-ai`, `CeError::Verification` maps to process exit code `6`. If `ce-ai gate check` were to return `CeError::Verification` on an unapproved write, Claude Code would treat it as a non-fatal warning, show stderr to the user in the UI, but **still execute the tool write**.
Therefore, to deterministically block write operations and provide immediate feedback to the agent model, `ce-ai gate check` MUST exit with code `2` (mapped through `CeError::Usage` or a hook-specific exit code contract).

### 1.2 Policy Table: Generalization vs Evidence-Based Rules
Issue #332 originally contemplated an elaborate 3-dimensional policy table mapping `(skill, stage, tier) -> required_artifacts`.
However, Issue #334 explicitly noted:
> *"El diseño concreto de la tabla de políticas... se termina de decidir con esos datos, no antes: si el spike solo detecta el caso `ce-work` + Stage 4... este issue empieza con una sola regla hardcodeada, no con la tabla generalizada de tres dimensiones que #332 proponía por adelantado."*

Based on live findings from Issue #333 and #337:
1. Stage 4 (`ce-work`) is the ONLY stage where production code writes under `src/**` occur and require formal OpenSpec contracts.
2. The only legitimate Stage 4 direct entry point that does not require full upfront OpenSpec is `ce-debug` (documented in `CLAUDE.md` and `workflow resume`).
3. Projects with `AdoptionTier::Minimal` explicitly do not enforce the full OpenSpec contract.
4. Edge cases (`mtime_fallback`, `worktree_uncommitted`, `stale_cycle_guard`) must remain non-blocking to prevent false-positive lockouts.

Therefore, a clean, targeted policy evaluation function meets all requirements without premature generalization.

### 1.3 Validation Receipt Storage Tradeoffs

| Option | Location | Pros | Cons | Decision |
|---|---|---|---|---|
| **Option A: Full DDW SLSA Matrix** | `openspec/changes/<feat>/*.validation.md` with multi-tier checkboxes | Comprehensive, human-readable markdown table | High token/I/O ceremony; requires complex markdown parser in `doctor` | Rejected: over-engineering for current needs |
| **Option B: Standalone State Only** | `state.json` (`state.gate_receipts`) | Fast O(1) query for `doctor` and `status` | Not committed with git repository; invisible to remote peers | Incomplete |
| **Option C: Dual Receipt (Selected)** | `openspec/changes/<feat>/.validation.json` + `state.json` + `gate-events.jsonl` | Git-committable receipt in feature directory, O(1) status lookup in `state.json`, and chronological audit trail in `gate-events.jsonl` | Requires keeping atomic writer safe | **Selected** |

### 1.4 Pre-Existing Bug in `probe_openspec_context_in`
When inspecting `src/commands/workflow.rs`, `probe_openspec_context_in` iterates `openspec/changes/*` looking for candidate feature folders when no git branch matches. In v1.52.0, PR #356 introduced `openspec/changes/archive/`. Because `archive` is a directory in `openspec/changes/`, its modification time was refreshed during archival, causing mtime fallback to falsely resolve `archive` as an active feature name (`active feature 'archive' resolved via mtime fallback`).
This must be fixed by filtering out `archive` and any directory starting with `.` from candidate feature names.

## 2. Evaluated Options

| Dimension | Option 1 | Option 2 (Selected) | Rationale |
|---|---|---|---|
| **Blocking Mechanism** | Stderr message with Exit Code 6 (`Verification`) | Stderr message with Exit Code 2 (`Usage`) | Option 2 is required by Claude Code's PreToolUse runtime specification to actually abort the tool call. |
| **Receipt Format** | Human-readable Markdown | Machine-readable `.validation.json` + state | Option 2 allows seamless serialization, schema validation, and integration with `doctor --json`. |
| **Policy Scope** | Check all files in repo | Check only `src/**` product files | Option 2 avoids blocking doc edits (`docs/`), specs (`openspec/`), or repo metadata. |
| **Mode Toggle** | Permanent hard blocking | Configurable (`mode: enforce \| observe`) with default `enforce` | Option 2 preserves backward compatibility and allows gradual adoption or debugging. |
