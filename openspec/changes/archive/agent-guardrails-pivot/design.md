# Technical Design: Agent Guardrails & Risk Engine Pivot (`ce-ai guard`)

## System Architecture

```text
┌────────────────────────────────────────────────────────────────────────┐
│                   Autonomous Agent / Tool Hook / CI                    │
│           (e.g., Claude Code, Antigravity, Background Worker)          │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                    ce-ai guard check <tool> <command>
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                  Stage 1: Deterministic Denial Engine                  │
│                        (Zero-latency, < 1 ms)                          │
│                                                                        │
│   Hard-block patterns:                                                 │
│   • sudo rm -rf /                                                      │
│   • git push --force (origin main/master)                              │
│   • Reading/exfiltrating ~/.ssh/, .env, AWS credentials                │
│   • Destructive pipe-to-bash (curl ... | sudo bash)                    │
└───────────────────┬────────────────────────────────┬───────────────────┘
                    │                                │
           Match Found?                              No Match
                    │                                │
                    ▼                                ▼
            Policy: DENY             ┌───────────────────────────────────┐
            (Exit Code 0,            │ Stage 2: Semantic Risk Evaluator  │
             Policy JSON)            │                                   │
                                     │ 6 Dimensions:                     │
                                     │ • destructive                     │
                                     │ • credential_sensitive            │
                                     │ • external_side_effect            │
                                     │ • privilege_escalation            │
                                     │ • irreversible                    │
                                     │ • scope_exceeds_task              │
                                     │                                   │
                                     │ Providers:                        │
                                     │ • Laya-MLX (Apple Silicon, <20ms) │
                                     │ • Kev (Universal Local)           │
                                     │ • Deterministic Fallback Policy   │
                                     └─────────────────┬─────────────────┘
                                                       │
                                                       ▼
                                     ┌───────────────────────────────────┐
                                     │ Composite Score & Threshold Gate  │
                                     │ • Score < 60%: Allow              │
                                     │ • Score 60-89%: Confirm           │
                                     │ • Score >= 90%: Deny              │
                                     └───────────────────────────────────┘
```

## CLI Interface Contracts

### 1. New Subcommand: `ce-ai guard`

```text
ce-ai guard [OPTIONS] <COMMAND>

Commands:
  check     Evaluate execution risk policy for a command or tool invocation
  status    Inspect Guardrails engine status, provider health, and active thresholds
  setup     Quick-setup preset configuration for Guardrails (laya, kev, local, off)
  policy    Inspect or configure risk threshold policies
  test      Run a self-test evaluation probe on sample commands
```

#### Detailed Command Contracts

* **`ce-ai guard check <TOOL> <COMMAND> [--task <TASK>] [--verbose] [--json]`**
  - Evaluates tool execution risk.
  - Returns structured text or machine-readable JSON:
    ```json
    {
      "tool": "run_command",
      "command": "git push --force origin main",
      "policy": "Deny",
      "composite_score_pct": 95,
      "reason": "Deterministic safety violation: force push to protected main/master branch",
      "latency_ms": 1,
      "fallback_applied": false
    }
    ```
  - Standard exit code: `0` (Success, with decision in stdout).

* **`ce-ai guard status [--json]`**
  - Displays whether guardrails are active, the active provider (`laya`, `kev`, `local`, `off`), and configured thresholds.

* **`ce-ai guard setup --preset <recommended|laya|kev|local|off>`**
  - `recommended`: Auto-detects platform. If macOS `aarch64`, selects `laya` with fallback to deterministic local rules. No cloud key required.
  - `laya`: Configures local MLX provider on `127.0.0.1:8008`.
  - `kev`: Configures local Kev provider on `127.0.0.1:8009`.
  - `local`: Configures offline deterministic baseline evaluation (zero daemons needed).
  - `off`: Disables guardrail checks.

### 2. Backward Compatibility & Deprecation Contracts

* **`ce-ai decisions check-risk`**:
  - Aliased directly to `ce-ai guard check`.
* **`ce-ai decisions route`**:
  - Emits deprecation notice:
    ```text
    ⚠️ Notice: 'ce-ai decisions route' is deprecated. Modern AI harnesses manage model
    selection natively via their own configuration files and CLI flags.
    ```
  - Exits with code `0`.
* **`ce-ai decisions compare`**:
  - Emits deprecation notice pointing to native harness metrics.

### 3. State Schema Alignment (`src/state/state.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_guard_provider")]
    pub provider: String,
    #[serde(default)]
    pub thresholds: RiskThresholds,
    #[serde(default)]
    pub fallback: RiskFallbackPolicy,
    #[serde(default)]
    pub laya: LayaConfig,
    #[serde(default)]
    pub kev: KevConfig,
}

fn default_guard_provider() -> String {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "laya".to_string()
    } else {
        "local".to_string()
    }
}
```

The existing `decisions` block in `state.json` is preserved and mapped seamlessly so existing projects experience zero migration churn.
