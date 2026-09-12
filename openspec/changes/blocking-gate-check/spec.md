# Specification: Blocking Gate Check & Validation Receipt (Issue #334)

## 1. Exit Code & Blocking Requirements

- **WHEN** `ce-ai gate check` evaluates a tool write in Stage 4 (`ce-work`) under `src/**` for an adopted project, AND the active feature is missing `proposal.md`, `spec.md`, or `tasks.md`, AND `gate_mode` is `enforce`,
  **THEN** `ce-ai gate check` MUST exit with process exit code `2`, print the blocking remediation message to `stderr`, append a record to `gate-events.jsonl` with `decision: "blocked"`, and write a `GateReceipt` record with `decision: "blocked"`.

- **WHEN** `ce-ai gate check` blocks a write operation,
  **THEN** the `stderr` output MUST specify:
  1. The target file path attempted.
  2. The active workflow stage and feature name.
  3. The exact list of missing OpenSpec contract artifacts (`proposal.md`, `spec.md`, `tasks.md`).
  4. Concrete remediation commands (e.g. running `/ce-plan` or checkpointing as `ce-debug`).

## 2. Policy Matrix & Exemption Requirements

- **WHEN** the tool call targets a file path outside `src/**` (such as `docs/**`, `openspec/**`, `README.md`, or project configuration files),
  **THEN** `ce-ai gate check` MUST pass with exit code `0` without evaluating OpenSpec contract artifacts.

- **WHEN** the project is adopted with `AdoptionTier::Minimal`,
  **THEN** `ce-ai gate check` MUST pass with exit code `0` and record `decision: "pass"` with reason stating tier minimal exemption.

- **WHEN** the active workflow task indicates the `ce-debug` direct entry point (the task text contains `ce-debug` or `debug`), OR `--entry-point ce-debug` is provided,
  **THEN** `ce-ai gate check` MUST pass with exit code `0` and record `decision: "pass"` with reason stating `ce-debug` bugfix exemption.

- **WHEN** the declared workflow stage is not Stage 4 (`WorkflowStage::WorkTdd`),
  **THEN** `ce-ai gate check` MUST pass with exit code `0` and record `decision: "pass"`.

- **WHEN** the gate check detects an edge-case condition (`mtime_fallback`, `worktree_uncommitted`, or `stale_cycle_guard`),
  **THEN** `ce-ai gate check` MUST pass with exit code `0`, MUST NOT block execution, and MUST log the specific `edge_case` category to `gate-events.jsonl`.

- **WHEN** all required OpenSpec contract artifacts (`proposal.md`, `spec.md`, `tasks.md`) are present and non-empty in `openspec/changes/<feature>/`,
  **THEN** `ce-ai gate check` MUST pass with exit code `0` and record `decision: "pass"`.

## 3. Validation Receipt Requirements

- **WHEN** `ce-ai gate check` completes an evaluation (either passing or blocking),
  **THEN** it MUST record a structured `GateReceipt`:
  1. Appending the event to `~/.ce-ai/gate-events.jsonl`.
  2. Updating `state.gate_receipts` in `state.json` keyed by feature name.
  3. If the directory `openspec/changes/<feature>/` exists on disk, writing an atomic receipt to `openspec/changes/<feature>/.validation.json`.

- **WHEN** `ce-ai status` is executed,
  **THEN** it MUST display the aggregated gate metrics including total observed, blocked writes, passes, and edge cases.

- **WHEN** `ce-ai doctor` is executed,
  **THEN** it MUST report gate check metrics, and if blocked writes exist, surface them as actionable warnings.

## 4. Emergency Kill-Switch & Configuration Requirements

- **WHEN** the emergency kill-switch is activated via `CE_AI_DISABLE_GATE_CHECK=1`, `CE_AI_GATE_CHECK_DISABLED=1`, or the `--disabled` CLI flag,
  **THEN** `ce-ai gate check` MUST immediately exit with code `0` without inspecting filesystem state, git status, or state files.

- **WHEN** `gate_mode` is set to `observe` (via CLI flag `--mode observe` or env var `CE_AI_GATE_MODE=observe`),
  **THEN** `ce-ai gate check` MUST NOT exit with non-zero exit codes upon missing contracts, but instead log `decision: "would_block"` and exit with code `0`.

## 5. Archive Folder Directory Exclusion Requirements

- **WHEN** `probe_openspec_context_in` scans `openspec/changes/` for candidate feature directories via mtime fallback,
  **THEN** it MUST ignore any directory named `archive` or starting with `.`, ensuring `archive` is never returned as the active feature name.
