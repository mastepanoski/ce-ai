# Proposal: Agent Guardrails & Risk Engine Pivot (`ce-ai guard`)

## Problem Statement

The **Pluggable Decision Engine** (`src/decisions/`, `ce-ai decisions`) was originally introduced as a multi-purpose System 1 classification layer handling four disparate domains:
1. **Model Routing (`route`, `compare`):** Probabilistically selecting between Fast, Standard, and High-Reasoning LLM classes.
2. **Dynamic Skill Routing (`skills resolve`):** Semantically resolving relevant skills via cloud classification queries.
3. **Work Readiness Advisory (`check-readiness`):** Probabilistically assessing whether a task or Compound Engineering stage DoD is met.
4. **Command Execution Risk (`check-risk`):** Evaluating destructive, irreversible, or credential-sensitive risks of tool invocations.

In the contemporary agentic landscape (2026), the foundational assumptions underlying this architecture have fundamentally shifted:
* **Harness Autonomy & Native Model Control:** Modern harnesses (Antigravity, Claude Code, Cursor, OpenCode, Codex) control model selection natively via their own configuration files and CLI flags. Following the decommissioning of `ce-ai models` in v3.0.0, `ce-ai decisions route` became an isolated advisory command with a hardcoded model catalog (`claude-3-5-*`) that no harness consumes automatically. Empirically, it overclassifies tasks to high-cost reasoning models with low confidence (~48%), yielding zero real-world cost savings.
* **Network Latency & Third-Party Friction:** The default preset (`recommended`) sets `provider: "jev"` (TypeSafe AI cloud API), introducing ~316–420 ms of network latency per micro-decision, tracking monthly spend against an external budget, and generating confusing `doctor-warn: decision-provider-unauthenticated` warnings in `ce-ai doctor` for developers who neither use nor require TypeSafe AI.
* **The Real Need in Autonomous Coding:** What autonomous agents and CI loops genuinely require is not advice on which model to use, but a fast, local-first, deterministic-precedence **Security Guardrail** to prevent destructive bash commands, credential exfiltration, and unauthorized force pushes before tool execution.

This change package pivots the Decision Engine subsystem into a dedicated, first-class **Agent Guardrails & Risk Engine (`ce-ai guard`)**, shedding obsolete routing code, dethroning cloud Jev as the default, and promoting local-first safety evaluation.

## Proposed Changes

1. **Promote First-Class `ce-ai guard` Subcommand:**
   - Introduce `ce-ai guard` in CLI command dispatch with dedicated actions:
     - `check`: Evaluate execution risk for tool invocations (`ce-ai guard check "<tool>" "<command>" [--task "<context>"]`).
     - `status`: Inspect guardrail engine health, active provider (local/MLX/remote), and threshold policies.
     - `policy`: Inspect and tune risk thresholds (`allow`, `require_confirmation`, `deny`).
     - `test`: Run hermetic execution risk evaluation probes.
   - Maintain a backward-compatible alias from `ce-ai decisions check-risk` to `ce-ai guard check`.

2. **Decommission Obsolete Model Routing:**
   - Deprecate/remove `ce-ai decisions route` and `ce-ai decisions compare`.
   - Remove obsolete `ModelRouter`, `ModelRoutingConfig`, and hardcoded `ModelClassCatalog` from `src/decisions/routing.rs`.
   - Eliminate dead comparison analytics from `src/decisions/analytics.rs`.

3. **Re-align Presets & Dethrone Cloud Jev:**
   - Update `recommended` setup preset to prioritize local execution:
     - On Apple Silicon macOS (`aarch64`): Auto-recommend or default to `laya-mlx` (<20 ms local latency).
     - On universal platforms: Default to deterministic baseline security rules or local `kev` provider.
     - Jev becomes an explicit opt-in provider (`ce-ai guard setup --provider jev`), not the default recommendation.
   - Update `ce-ai doctor`: Eliminate unauthenticated warnings for Jev when the user has not explicitly configured TypeSafe AI.

4. **Reinforce Deterministic Safety & Fast Heuristics:**
   - Keep deterministic security precedence as the primary invariant: hardcoded blocks on catastrophic operations (`rm -rf /`, `.ssh` key access, `git push --force origin main`) execute in < 1 ms without external dependencies.
   - Streamline semantic risk dimension scoring (6 dimensions: destructive, credential_sensitive, external_side_effect, privilege_escalation, irreversible, scope_exceeds_task) for tool wrappers and autonomous agent hooks.

5. **Governance & Documentation Alignment:**
   - Update `CONCEPTS.md` to reflect the shift from a broad Decision Engine to the Agent Guardrails & Risk Engine.
   - Update `docs/user-guide/decision-engine-guide.md` (renamed or refactored to `agent-guardrails-guide.md`) and `README.md`.

## In-Scope vs. Out-of-Scope

### In-Scope
- `src/commands/guard.rs`: New top-level CLI command and action handlers.
- `src/commands/registry.rs` & `src/main.rs`: Wiring `Commands::Guard` and deprecating routing subcommands in `decisions`.
- `src/decisions/risk.rs`: Streamlining risk evaluator and pre-execution policies.
- `src/decisions/routing.rs`: Decommissioning obsolete model routing logic.
- `src/commands/doctor.rs`: Aligning guardrail diagnostic probes and silencing unauthenticated Jev false-positives.
- Updating `state.json` schema (`guard` / `decisions` backward compatibility).
- Full verification gates (`cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `make e2e`).

### Out-of-Scope
- Complete deletion of `LayaMlxProvider` or `KevProvider` (retained as local engines for semantic risk scoring).
- Changes to multi-harness fleet governance (`ce-ai fleet`) or project adoption (`ce-ai init-prj`).
- Modification of Compound Engineering 7-stage cycle FSM.

## Success Criteria

1. `ce-ai guard check "run_command" "git push --force origin main"` returns `RequireConfirmation` or `Deny` with deterministic precedence in < 5 ms.
2. Fresh installations of `ce-ai` and `ce-ai doctor` do not emit warnings about missing `TYPESAFE_API_KEY`.
3. `ce-ai decisions route` is cleanly deprecated with informative guidance pointing to native harness configuration.
4. All existing tests pass, and new comprehensive unit and integration tests cover `ce-ai guard`.
