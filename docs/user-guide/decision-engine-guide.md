# Agent Guardrails & Decision Engine Guide

The `ce-ai guard` subsystem (formerly `ce-ai decisions`) governs the **Agent Guardrails & Risk Execution Engine** — a fast, structured micro-decision layer providing deterministic safety precedence (<1 ms) and semantic risk scoring for autonomous agent tool invocations, alongside readiness checking and pluggable evaluation providers.

---

## 1. Quick Orientation & Audience Routing

| Goal | Persona | Action / Section |
| :--- | :--- | :--- |
| **Newbie / Setup** | Operator setting up credentials and enabling safe defaults | Jump to [Getting Started: Setup & Auth](#3-getting-started-setup--authentication) |
| **Developer / CI** | Inspecting risk, readiness, or switching runtime modes | Jump to [Operational Workflows](#5-operational-workflows) & [Mode Management](#4-runtime-mode-management) |
| **Senior / Architecture** | Auditing budget ceilings, circuit breakers, and determinism | Jump to [Safety & Resilience Invariants](#7-safety--resilience-invariants) |

---

## 2. Presets vs. Operational Modes Explained

A common point of confusion is the distinction between **Presets** (`--preset`) and **Operational Modes** (`mode`). They operate at different lifecycle levels:

```
+-------------------------------------------------------------------------+
| CLI Setup Preset (Template):                                            |
|   ce-ai guard setup --preset <recommended|shadow|kev|laya|local|off>    |
|   (or alias: ce-ai decisions setup --preset <...>)                      |
+-------------------------------------------------------------------------+
                              │
                              ▼ writes to state.json
+-------------------------------------------------------------------------+
| Runtime Operational Mode:                                               |
|   ce-ai decisions mode <active|shadow|off>                              |
+-------------------------------------------------------------------------+
```

### The Matrix

| Concept | Option | Provider | Mode | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Setup Preset** | `recommended` | `laya` / `local` | `active` | Local-first evaluation: uses ultra-low latency (<20ms) Laya MLX on Apple Silicon; uses deterministic local rules on universal platforms (zero cloud spend or keys required). |
| | `shadow` | `jev` | `shadow` | Evaluates in background to monitor latency and telemetry without altering deterministic policies. |
| | `laya` / `mlx` | `laya-mlx` | `active` | Ultra-low latency (<20ms) local Apple Silicon MLX inference daemon (`127.0.0.1:8008`). |
| | `kev` | `kev` | `active` | Local System 1 inference via Kev server (`127.0.0.1:8009`). Zero cost, unmetered, universal platform support. |
| | `local` | `mock` | `active` | 100% offline Mock provider for hermetic testing or environments without API keys. |
| | `off` / `disabled` | *none* | `off` | Completely disables the Decision Engine; zero network calls and zero evaluations. |
| **Runtime Mode** | `active` | *configured* | `active` | Evaluations actively enforce policies (e.g., risk confirmation, model routing). |
| | `shadow` | *configured* | `shadow` | Evaluations run in background; results are logged but do not block or alter execution. |
| | `off` | *configured* | `off` | Engine is dormant. Core execution relies purely on deterministic rules. |

---

## 3. Getting Started: Setup & Authentication

### Step 1: Apply a Configuration Preset

To provision recommended local-first defaults (Laya MLX on Apple Silicon, local deterministic rules on universal platforms):

```bash
ce-ai guard setup --preset recommended
# or via legacy alias:
ce-ai decisions setup --preset recommended
```

For local, zero-cost System 1 evaluation with Kev (universal):

```bash
ce-ai guard setup --preset kev
```

For ultra-low latency Apple Silicon evaluation with Laya-MLX:

```bash
ce-ai guard setup --preset laya
```

For offline or hermetic local testing with canned/mock responses:

```bash
ce-ai guard setup --preset local
```

To turn off evaluations entirely:

```bash
ce-ai guard setup --preset off
```

### Step 2: Configure Credentials (Remote) or Start Daemon (Local)

#### For Cloud Jev (Optional / `shadow`)
`ce-ai` never stores API keys in git-tracked files (`state.json`, `ce-ai.toml`). Keys are resolved in order:
1. Environment variable: `TYPESAFE_API_KEY` (or fallback `JEV_API_KEY`)
2. Local secured file: `~/.config/ce-ai/credentials.toml` (Unix permissions `0600`)
3. OS Keyring (macOS Keychain, Linux Secret Service, Windows Credential Manager)

Configure credentials interactively (masked input without terminal echoing):

```bash
ce-ai decisions auth --key
```

Or pass via standard input (ideal for CI/CD or secrets managers):

```bash
echo "$TYPESAFE_API_KEY" | ce-ai decisions auth --stdin
```

Verify provider connectivity and latency:

```bash
ce-ai decisions auth --check
```

#### For Local Kev (`kev`)
Kev executes unmetered without API keys. Ensure Kev is running locally on the configured port (default `8009`):

```bash
# In your Python / virtualenv environment:
pip install kev
python -m kev.serve --run jaredpalmer/kev-4b --port 8009
```

Verify connectivity:

```bash
ce-ai decisions test
```

#### For Local Laya-MLX (`laya`)
Laya provides sub-20ms MLX evaluation on Apple Silicon Macs (macOS `aarch64`):

```bash
# Start Laya MLX service on port 8008
laya serve --port 8008
```

Verify connectivity:

```bash
ce-ai decisions test
```

### Step 3: Inspect Engine Status & Health

View provider health, active preset, endpoint, monthly spend, and session limits:

```bash
ce-ai decisions status
```

For JSON output in automation scripts:

```bash
ce-ai decisions status --json
```

---

## 4. Runtime Mode Management

You can inspect or switch the operational mode at any time without resetting your budget or provider configuration:

### Check Current Mode

```bash
ce-ai decisions mode
# Output: Decision Engine mode: active (provider: laya / local)
```

### Toggle Operational Mode

```bash
# Switch to shadow mode (evaluates in background, zero enforcement)
ce-ai decisions mode shadow

# Switch to active mode (evaluations actively guide execution)
ce-ai decisions mode active

# Temporarily disable all evaluations
ce-ai decisions mode off
```

---

## 5. Operational Workflows

### 1. Risk-Aware Tool Execution (`ce-ai guard check`)

Before running potentially dangerous shell commands or tool invocations in autonomous loops, evaluate execution risk across deterministic safety rules (<1 ms) and 6 semantic dimensions (`destructive`, `credential_sensitive`, `external_side_effect`, `privilege_escalation`, `irreversible`, `scope_exceeds_task`):

```bash
ce-ai guard check "run_command" "git push --force origin main" --task "Deploy release"
# Backward-compatible alias:
ce-ai decisions check-risk "run_command" "git push --force origin main"
```

Output:
```text
Agent Guardrail Policy: DENY (Categorically blocked)
  Tool:       run_command
  Command:    git push --force origin main
  Score:      100.0%
  Reason:     Deterministic denial: Force push to protected branch detected
  Latency:    0ms
```

Policies returned:
- `Allow`: Risk score < 60%. Proceeds automatically.
- `RequireConfirmation`: Risk score between 60% and 89%. Prompts operator for approval.
- `Deny`: Risk score ≥ 90% or matches deterministic safety violations. Blocked immediately.

Add `--verbose` to inspect individual dimension scores or `--json` for programmatic consumption by pre-tool hooks.

### 2. Work Readiness Advisory (`check-readiness`)

Evaluate whether a task Definition of Done (DoD) or a Compound Engineering stage checkpoint is satisfied:

```bash
ce-ai decisions check-readiness --stage 4 --task "Implement OAuth token rotation"
```

Output:
```text
Readiness Advisory: ✓ Ready (88%)
Summary: DoD verification criteria met across code, unit tests, and security boundaries.
```

### 3. Model Routing (`route` — Deprecated)

> [!NOTE]
> `ce-ai decisions route` is deprecated in favor of native coding-agent harness configuration. Modern harnesses (Antigravity, Claude Code, Cursor, OpenCode) configure model selection natively through CLI flags or configuration files.

---

## 6. CLI Command Reference

| Command | Purpose | Key Flags |
| :--- | :--- | :--- |
| `ce-ai guard check` | Evaluate tool execution risk & safety policy | `<tool>`, `<command>`, `--task`, `--verbose`, `--json` |
| `ce-ai guard status` | Inspect guardrail operational status | `--json` |
| `ce-ai guard setup` | Configure guardrails preset | `--preset <recommended\|laya\|kev\|local\|off>` |
| `ce-ai guard test` | Run self-test evaluation probe | *none* |
| `ce-ai decisions setup` | Provision preset configurations | `--preset <recommended\|shadow\|local\|off>` |
| `ce-ai decisions mode [mode]` | Query or switch execution mode | `active`, `shadow`, `off` |
| `ce-ai decisions auth` | Manage and verify credentials | `--key`, `--stdin`, `--check` |
| `ce-ai decisions status` | Inspect provider health & budget | `--json` |
| `ce-ai decisions test` | Run hermetic evaluation probe | `--provider <jev\|mock>` |
| `ce-ai decisions check-risk` | Alias for `ce-ai guard check` | `--task`, `--verbose`, `--json` |
| `ce-ai decisions check-readiness` | Assess stage or task DoD readiness | `--stage`, `--task`, `--verbose`, `--json` |
| `ce-ai decisions route <task>` | *(Deprecated)* Model recommendation | positional `<task>`, `--verbose`, `--json` |

---

## 7. Safety & Resilience Invariants

The Decision Engine is engineered with strict defense-in-depth guarantees:

1. **Deterministic Security Precedence**: Probabilistic evaluations can *elevate* risk, but never lower it below deterministic baselines. Operations matching hardcoded violations (e.g. `sudo rm -rf /`, `/etc/shadow` tampering, `.ssh/id_rsa` exfiltration) receive immediate `Deny` regardless of provider output.
2. **Strict Budget Ceilings**: Accumulated API spend is tracked in a local sliding ledger. If `max_monthly_usd` (default: $5.00) or `max_session_requests` (default: 100) is reached, the circuit breaker opens.
3. **Fail-Closed Conservative Fallbacks**: If the external decision provider times out (>1000ms), encounters HTTP 5xx errors, or trips the circuit breaker:
   - Risk checks fail-closed to `RequireConfirmation` (never silently allowed).
   - Readiness checks fail-safe to `Needs Attention`.
   - The CLI never crashes or fails user commands (exit code 0 preserved).
4. **Credential Privacy**: API keys, auth tokens, and secret strings are automatically redacted (`[REDACTED]`) before being sent to evaluation payloads or recorded in `risk-events.jsonl`.
