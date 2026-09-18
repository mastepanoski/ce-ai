# Exploration: Risk-Aware Tool Execution (Intelligent Permission & Risk Engine)

## Architectural Investigation & Tradeoffs

### 1. Evaluated Architectural Approaches

| Approach | Latency | Context Awareness | Security Robustness | Cost / Overhead |
| :--- | :--- | :--- | :--- | :--- |
| **A. Pure Static Regex / Blocklist** | < 1ms | None (brittle) | Low (easily bypassed via aliases/subshells) | Zero |
| **B. Frontier LLM (System 2) per Tool Call** | 800–2,500ms | High | High (prompt injection risk) | Prohibitive (~500+ tokens/op) |
| **C. Hybrid 2-Tier: Deterministic Rules + System 1 Micro-Decisions (Chosen)** | < 1ms (rules) / ~20–50ms (Jev) | High | Absolute hard boundaries + probabilistic nuance | Minimal (micro-decision pennies) |

#### Tradeoff Analysis:
- **Why Approach A Fails**: A static blocklist cannot distinguish `git clean -fd` within a scratch directory from deleting untracked source files. Similarly, `rm -rf build/` is routine, whereas `rm -rf src/` is dangerous.
- **Why Approach B Fails**: Calling an LLM like Claude 3.5 Sonnet or GPT-4o for every single tool invocation adds hundreds of milliseconds of latency to every agent turn and inflates token bills rapidly.
- **Why Approach C Wins**: Approach C enforces a strict defense-in-depth model:
  1. Deterministic rules execute in microseconds: protected paths, credentials, and known exploit patterns are categorically rejected without network calls.
  2. Safe read-only commands (e.g. `ls`, `git status`, `cat`) are immediately allowed.
  3. Only ambiguous, state-mutating actions trigger a lightweight System 1 micro-decision query.
  4. Probabilistic outputs can only elevate caution (e.g. `Allow` -> `RequireConfirmation`), never weaken a deterministic rejection.

---

### 2. Risk Dimension Modeling

To ensure accurate, multi-faceted risk assessment without vendor-specific prompt lock-in, the risk engine models six orthogonal dimensions:

1. **`destructive`**: File deletions, truncations, database drops, branch deletions.
2. **`credential_sensitive`**: Accessing `.env`, SSH keys, credentials files, bearer tokens, or environment exports.
3. **`external_side_effect`**: Outbound HTTP requests, external deployments, webhooks, cloud service manipulations.
4. **`privilege_escalation`**: Executing as `root`, `sudo`, `doas`, or modifying system file permissions (`chmod`, `chown`).
5. **`irreversible`**: Actions whose consequences cannot be recovered via Git history or local backups.
6. **`scope_exceeds_task`**: Operations completely extraneous to the stated user task prompt.

Each dimension is queried as a boolean probability (`DecisionQuestion::Boolean`), alongside an overall risk classification choice (`DecisionQuestion::Choice: ["safe", "sensitive", "destructive"]`).

---

### 3. Redaction & Data Privacy Strategy

Before transmitting commands or tool arguments to the Decision Provider or writing to audit logs:
- API keys matching common prefixes (`sk-`, `ghp_`, `ts_`, etc.) are replaced with `[REDACTED_API_KEY]`.
- Key-value pairs matching `(?i)(password|secret|token|apikey|api_key)\s*[:=]\s*(\S+)` are replaced with `[REDACTED]`.
- Private key headers (`-----BEGIN ... PRIVATE KEY-----`) are redacted.

---

### 4. Deterministic State & Configuration Invariants

To comply with `ce-ai`'s architectural invariants:
- **`Eq` trait compliance in `State`**: Risk thresholds are defined as `u32` integer percentages:
  - `confirmation_threshold_pct: u32 = 60` (60%)
  - `deny_threshold_pct: u32 = 90` (90%)
- **Atomic persistence**: `state.json` modifications continue using `write_atomic`.
- **Fail-closed default**: In the event of provider timeout, HTTP error, or circuit-breaker trip, the fallback policy defaults to `ExecutionPolicy::RequireConfirmation`.
