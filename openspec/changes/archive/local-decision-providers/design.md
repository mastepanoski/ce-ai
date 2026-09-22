# Design: Local Decision Engine Providers (Kev & Laya-MLX)

## Architecture Overview

`ce-ai`'s Decision Engine abstracts providers through the `DecisionProvider` trait in [`src/decisions/mod.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/decisions/mod.rs):

```
+-------------------------------------------------------------+
|                     Decision Engine                         |
|  (Adaptive Routing, Skill Routing, Risk Gate, Readiness)    |
+-------------------------------------------------------------+
                              |
                   [DecisionProvider Trait]
                              |
      +---------------+-------+-------+---------------+
      |               |               |               |
[JevProvider]   [KevProvider]   [LayaMlxProvider]  [MockProvider]
(Cloud API)     (Local/Kev)     (Local/MLX Mac)     (Testing)
      |               |               |               |
 TypeSafe Cloud  Local HTTP       Local Daemon    In-memory
 (api.typesafe) (:8009/v1/sys1)   (:8080 or UDS)  canned data
```

---

## 1. Structs & Data Schemas

### A. State Configuration (`src/state/state.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KevConfig {
    /// Kev server endpoint (default: "http://127.0.0.1:8009/v1").
    pub endpoint: String,
    /// Model identifier (default: "kev-latest", "jaredpalmer/kev-4b", etc.).
    pub model: String,
    /// Request timeout in milliseconds (default: 2000).
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayaConfig {
    /// Laya daemon endpoint (default: "http://127.0.0.1:8080/v1").
    pub endpoint: String,
    /// Optional Unix Domain Socket path for sub-millisecond IPC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket_path: Option<String>,
    /// Model checkpoint identifier (default: "aac6fef/laya-mlx").
    pub model: String,
    /// Request timeout in milliseconds (default: 250).
    pub timeout_ms: u64,
}

// In DecisionsConfig:
pub struct DecisionsConfig {
    pub enabled: bool,
    pub provider: String, // "jev" | "kev" | "laya" | "mock"
    pub mode: DecisionMode,
    pub budget: BudgetConfig,
    pub jev: JevConfig,
    #[serde(default)]
    pub kev: KevConfig,
    #[serde(default)]
    pub laya: LayaConfig,
    // ...
}
```

### B. Wire Protocol Normalization

Both Kev and Laya follow the TypeSafe System One schema:
```json
// POST /v1/systemone
{
  "state": "...",
  "model": "...",
  "questions": {
    "question_id": {
      "type": "choice | noul | score",
      "instructions": "...",
      "criteria": ...
    }
  }
}
```

Domain translation logic:
- `DecisionQuestion::Boolean { id, question }` maps to `{"type": "noul", "instructions": question}`.
- `DecisionQuestion::Choice { id, question, options }` maps to `{"type": "choice", "instructions": question, "criteria": { option: null }}`.
- `DecisionQuestion::Score { id, question, min, max }` maps to `{"type": "score", "instructions": question, "criteria": [...]}`.

Response mapping:
- `noul` probability $P$ maps to `DecisionAnswer::Boolean { value: P >= 0.5, confidence: (P - 0.5).abs() * 2.0 }`.
- `choice` maps to `DecisionAnswer::Choice { selected: choice, confidence, probabilities }`.
- `score` maps to `DecisionAnswer::Score { score, confidence }`.

---

## 2. CLI & Ergonomics

### Setup Command Presets
```bash
# Setup with Kev (universal local provider)
ce-ai decisions setup --preset kev
ce-ai decisions setup --provider kev --endpoint http://127.0.0.1:8009/v1

# Setup with Laya (Apple Silicon high-speed provider)
ce-ai decisions setup --preset laya
ce-ai decisions setup --provider laya --endpoint http://127.0.0.1:8080/v1
```

### Connectivity Test
```bash
ce-ai decisions test
# Output:
# Testing Decision Provider: kev (http://127.0.0.1:8009/v1)
# Health Check: Healthy (Kev-4B bf16, latency 12ms)
# Sample Decision: Pass (Choice returned 'returns' with confidence 0.88 in 142ms)
```

### Doctor Integration (`ce-ai doctor`)
Probes the configured provider:
- `kev`: Queries `GET {endpoint}/models`. If failed, outputs:
  ```
  [!] Kev local server unreachable at http://127.0.0.1:8009.
      Start with: uv run --extra serve python -m kev.serve --run jaredpalmer/kev-4b --port 8009
  ```
- `laya`: Checks whether platform is `aarch64-apple-darwin`. If not:
  ```
  [x] laya-mlx requires macOS Apple Silicon (aarch64). Current OS: linux-x86_64.
      Recommended: Run 'ce-ai decisions setup --provider kev' instead.
  ```
  If on Apple Silicon, probes socket/HTTP endpoint and reports measured latency (< 20ms expected).
