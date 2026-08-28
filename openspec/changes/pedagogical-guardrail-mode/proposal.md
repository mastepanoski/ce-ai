# Proposal: Pedagogical Guardrail Mode (`ce-ai guard`)

## Problem Statement & Why

Junior developers using AI coding agents increasingly fall into **vibe coding**: integrating generated code without understanding its mechanics, architecture, or edge cases. This introduces invisible technical debt, weakens engineering competence, and undermines effective human oversight required by our AI governance commitments (`AI_POLICY.md` — ISO/IEC 42001, NIST AI RMF 1.0).

Currently, `ce-ai` governs the 7-stage compound engineering lifecycle across 11 AI harnesses but lacks a dedicated mechanism to enforce developer comprehension, structured Human-In-The-Loop (HITL) checkpoints, and didactic explain-back loops.

Issue reference: [Issue #114](https://github.com/mastepanoski/ce-ai/issues/114).

## What Changes

1. **Lifecycle Subcommands (`src/commands/guard.rs`):**
   - `ce-ai guard enable [--level junior|strict] [--harness <name>]`: Opt-in activation of pedagogical guardrails, persisted in `state.json`.
   - `ce-ai guard disable [--harness <name>]`: Clean, surgical deactivation restoring pre-enable configuration.
   - `ce-ai guard status [--json]`: Machine-readable and human-formatted status report with SHA256 integrity checks.

2. **State & Schema Extensions (`src/state/state.rs`):**
   - New `GuardrailState` and `GuardLevel` (`Junior`, `Strict`) enums/structs in `state.json`.
   - Backward-compatible deserialization (`#[serde(default)]`, `#[serde(skip_serializing_if = "Option::is_none")]`).
   - Mutation protected by atomic file writes (`crate::state::write_atomic`).

3. **Governance & Health Check Integration (`src/commands/doctor.rs`):**
   - `doctor` reports `Guardrail: enabled (junior/strict)` or `disabled`.
   - Manifest drift detection: alerts if pedagogical assets have been silently bypassed or modified.

4. **TUI Parity (`src/tui/`):**
   - Status tab and Command Registry include `Guard` lifecycle commands.
   - Interactive toggle / status display within TUI dashboard.

5. **Managed Pedagogical Assets:**
   - Managed pedagogical skill definitions providing structured 2-checkpoint HITL gating (Spec Approval and Architectural Tradeoffs) and lightweight explain-back validation.

## Scope Boundaries

- **In-Scope:**
  - CLI commands `ce-ai guard enable`, `ce-ai guard disable`, `ce-ai guard status`.
  - Schema extension in `state.json` with full round-trip backward compatibility.
  - Integration with `doctor`, `status`, and `tui`.
  - SHA256 manifest drift detection for guardrail assets.
  - `--dry-run` and `--json` output support.

- **Out-of-Scope (Non-Goals):**
  - Cryptographic execution blocking of external IDE processes (governance is asset/contract-driven).
  - Overwriting unmanaged user customizations or custom skills in `opencode.json`.
  - Mandatory global enforcement without opt-in (must remain explicitly opt-in).

## Risk Register (ISO/IEC 42001 & NIST AI RMF Aligned)

| Risk ID | Description | Severity | Control Reference | Mitigation Strategy |
|:---|:---|:---|:---|:---|
| **R1** | **Rubber-stamping:** Junior developer blindly approves prompts without reading | High | Annex A.8 Accountable Use & Effective Human Oversight | Mandatory explain-back questions in spec stage; engagement visibility in `doctor`. |
| **R2** | **Excessive friction:** Flow interruptions cause developers to disable the mode | High | Clause 8 Operational Planning | Risk-tiered gating: only 2 hard checkpoints; mechanical tasks auto-approved. |
| **R3** | **Silent bypass:** Guardrail files deleted or tampered | Medium | Annex A.6 Asset Integrity | SHA256 manifest drift detection via `sync` and surfaced by `doctor`. |
| **R4** | **Configuration clobbering:** Activating guardrail overwrites user custom skills | Medium | ISO 27002 Config Management | Non-destructive JSON merging with timestamped backups via `write_atomic`. |
| **R5** | **Token cost inflation:** Verbose tutoring expands context windows excessively | Low | NIST AI RMF Resource Efficiency | Surgical context injection; regeneration caps; efficiency auditing. |

## Success Criteria

1. `ce-ai guard enable --level junior|strict` updates `state.json` atomically and registers pedagogical assets.
2. `ce-ai guard disable` cleanly removes guardrail configuration and restores pristine state.
3. `ce-ai guard status` and `ce-ai doctor` transparently report mode, level, and asset integrity.
4. 100% backward compatibility: existing workspaces without `guardrail` field deserialize without error.
5. All verification gates pass (`cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `make e2e`).
