# Proposal: Spike Observe-Only — Medir (Sin Bloquear) Escritura en ce-work Sin OpenSpec Aprobado

## 1. Problem Statement

Issue #332 originally proposed a blocking gate (hook + validation receipt) to prevent code writes when OpenSpec contracts are absent. However, design review revealed a critical foundational flaw: **there is zero measured empirical evidence that the gap exists**. The only evidence cited in #332 (`29 OpenSpec change(s) complete but not archived`) demonstrated the *opposite* phenomenon: specs completed without being archived, rather than unvetted code written without an approved spec.

Attempting to enforce binary blocking without knowing the real false-positive rate risks breaking legitimate, sanctioned developer workflows (such as `ce-debug` acting as the *"Direct Entry Point for Bug Fixes"*). As documented in `docs/solutions/architecture/pedagogical-guardrail-mode-lifecycle-2026-08-28.md` (Issue #114) and historical DDW post-mortems, interceptors that lack empirical calibration inevitably generate friction, causing users to bypass guards entirely.

This proposal implements **Issue #333**: an **observe-only spike** that monitors agent write activity under `src/**` in Claude Code during Stage 4 (`ce-work`), evaluates whether the active task has approved OpenSpec artifacts (`proposal.md`, `spec.md`, `tasks.md`), records structured telemetry, and **never blocks any write**. This empirical data will determine whether the future blocking gate (Issue #334) is justified and how its policy rules should be shaped.

## 2. In-Scope / Out-of-Scope Boundaries

### In-Scope:
1. **Pure Decision Engine (`src/commands/gate.rs`)**:
   - Pure, deterministic, side-effect-free function: `evaluate_gate_decision(declared_stage, feature, resolution, is_new_cycle, is_uncommitted, artifacts) -> (GateDecision, String)`.
   - Reads declared stage directly from active `WorkflowState` checkpoint in `state.rs` — **never re-infers stage**.
2. **Four Distinct Decision Classes**:
   - `would-block`: Stage 4 (`ce-work`) active, but missing one or more required artifacts (`proposal.md`, `spec.md`, `tasks.md`) for the active task/feature.
   - `pass`: Valid OpenSpec artifacts present in Stage 4, or declared stage is not Stage 4, or non-target write.
   - `undetermined`: Checkpoint is missing, ambiguous, or corrupted (e.g., non-adopted workspace).
   - `edge_case`: Separated into three discrete buckets (Issue #337):
     - `mtime_fallback`: `resolution == FeatureResolution::MtimeFallback`.
     - `worktree_uncommitted`: `probe_openspec_has_uncommitted` detects dirty/uncommitted spec.
     - `stale_cycle_guard`: `task.contains("(nuevo ciclo detectado)")` / inferred cycle reset.
     *These edge case buckets are never conflated with happy-path `would-block` or `pass`.*
3. **Emergency Kill-Switch**:
   - Environment variable `CE_AI_DISABLE_GATE_CHECK=1` (or `CE_AI_GATE_CHECK_DISABLED=1`) and CLI flag `--kill-switch` / `--disabled`.
   - When active, `ce-ai gate check` performs zero evaluation and zero I/O (absolute no-op). Implemented first.
4. **CLI Subcommand `ce-ai gate check`**:
   - Accepts tool and path via explicit CLI arguments (`--tool`, `--path`) and via Claude Code `PreToolUse` JSON payload on stdin.
   - Always exits with code `0`. Execution is never blocked.
5. **Structured Telemetry Logging**:
   - Append-only JSONL log at `~/.ce-ai/gate-events.jsonl` recording timestamp, harness, tool, path, stage, feature, decision, and rationale.
   - Zero write content is logged (paths and decisions only).
6. **Observability Surfacing in `doctor` and `status`**:
   - Displays aggregated counts (`would-block`, `pass`, `undetermined`, and edge cases) without exposing file contents.
7. **Thin Claude Code Hook Integration (`src/harness/claude.rs`)**:
   - Configures Claude Code `PreToolUse` hook in `.claude/settings.json` targeting `Write` and `Edit` tools.
   - Hook script acts strictly as a thin invocation wrapper for `ce-ai gate check`.

### Out-of-Scope:
- Blocking or intercepting writes (Issue #334 remains blocked pending telemetry).
- Synthesizing or fabricating telemetry data.
- Closing the 2-week observation window (this PR delivers the mechanism; evaluation is a future operational step).
- Harnesses other than Claude Code for this spike.

## 3. Risk Evaluation

- **Workflow Interruption Risk**: **Zero**. `ce-ai gate check` always exits `0`, and errors during logging are safely ignored without disrupting tool execution.
- **Fail-Safe Deactivation**: The kill-switch env var `CE_AI_DISABLE_GATE_CHECK=1` short-circuits execution before any filesystem or state inspection.
- **Performance Overhead**: Execution is strictly bounded: reading local JSON state and appending a single line to a local log file takes < 3ms.
- **Privacy & Data Security**: No code modifications, buffers, or file contents are written to the log; only metadata (tool name, file path, stage, decision).

## 4. Success Criteria

1. 100% unit test coverage on the pure decision function for all combinations of stages, missing artifacts, and edge cases.
2. CLI tests verify `ce-ai gate check` returns `0` under all scenarios (missing spec, corrupt state, kill-switch active).
3. `gate-events.jsonl` accurately logs structured decisions with separated edge cases.
4. `ce-ai status` and `ce-ai doctor` cleanly present gate telemetry metrics.
5. Full compliance with DoD: zero clippy warnings, clean formatting, green unit & CLI tests.
