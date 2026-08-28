---
module: guard
tags: [guardrail, hitl, pedagogical, iso-42001, nist-ai-rmf, anti-vibe-coding]
problem_type: architecture
---

# Pedagogical Guardrail Mode Lifecycle (`ce-ai guard`)

## Problem
In developer workflows incorporating AI coding assistants (Compound Engineering, Spec-Driven Development), junior or inexperienced practitioners often fall into "vibe-coding" anti-patterns: accepting AI proposals without reading diffs, approving plans without understanding architectural trade-offs, and bypassing specification phases. This violates **ISO/IEC 42001** and **NIST AI RMF 1.0** human-in-the-loop oversight principles and leads to regression risks.

## Solution
Implemented **Issue #114 — Pedagogical Guardrail Mode** via a dedicated CLI subcommand group (`ce-ai guard`) and schema extension in `state.json`:

1. **State Persistence & Schema Backward Compatibility (`src/state/state.rs`):**
   - Added `GuardLevel` enum (`junior` for batched oversight, `strict` for per-module checkpoints).
   - Added `GuardrailState` struct (`enabled`, `level`, `harness`, `updated_at`).
   - Extended `State` with `#[serde(default, skip_serializing_if = "Option::is_none")] pub guardrail: Option<GuardrailState>`, guaranteeing 100% roundtrip compatibility with legacy state files.

2. **CLI Lifecycle Commands (`src/commands/guard.rs`):**
   - `ce-ai guard enable [--level junior|strict] [--harness <name>]`: Enables guardrail mode atomically with `--dry-run` support.
   - `ce-ai guard disable [--harness <name>]`: Cleanly disables guardrail mode.
   - `ce-ai guard status [--json]`: Reports active status, level, scope, and update timestamp in plain text or structured JSON.

3. **System Visibility & Governance:**
   - Integrated into `ce-ai doctor` as an informative health probe.
   - Integrated into `ce-ai status` and interactive TUI status rendering.
   - Dispatched via Strategy trait `CeCommand` in `src/commands/registry.rs`.

## Key Learnings
1. **Separation of Concerns:** The CLI governs the state and lifecycle, while skills enforce didactic flow, avoiding binary interception bottlenecks.
2. **Deterministic Exit Codes:** Strict adherence to exit code 2 on invalid CLI arguments (e.g. unsupported level) prevents silent error propagation.
3. **Atomic Mutations:** Utilizing `State::save` via `write_atomic` prevents race conditions or corrupted configurations during CLI mutations.
