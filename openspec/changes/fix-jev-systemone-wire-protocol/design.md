# Design: TypeSafe System One Wire Protocol Sharing & Jev Migration

## 1. System Architecture
```
                      ┌──────────────────────────────────────────────┐
                      │          src/decisions/types.rs              │
                      │  - SystemOneWireQuestion                     │
                      │  - SystemOneWireRequest                      │
                      │  - SystemOneWireAnswer                       │
                      │  - SystemOneWireResponse                     │
                      │  - helper: build_systemone_questions()       │
                      │  - helper: parse_systemone_answers()         │
                      └──────────────┬───────────────────────────────┘
                                     │
         ┌───────────────────────────┼───────────────────────────┐
         ▼                           ▼                           ▼
┌──────────────────┐       ┌──────────────────┐       ┌──────────────────┐
│    jev.rs        │       │    kev.rs        │       │    laya.rs       │
│ POST /systemone  │       │ POST /systemone  │       │ POST /systemone  │
│ Bearer auth      │       │ Local HTTP       │       │ Apple Silicon MLX│
└──────────────────┘       └──────────────────┘       └──────────────────┘
```

## 2. Wire Protocol Structs (`src/decisions/types.rs`)
```rust
/// System One wire question schema accepted by TypeSafe AI System One compatible engines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOneWireQuestion {
    #[serde(rename = "type")]
    pub question_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<serde_json::Value>,
}

/// System One wire request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOneWireRequest {
    pub state: serde_json::Value,
    pub model: String,
    pub questions: BTreeMap<String, SystemOneWireQuestion>,
}

/// System One wire answer schema returned by TypeSafe AI System One compatible engines.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemOneWireAnswer {
    #[serde(rename = "type", default)]
    pub answer_type: Option<String>,
    #[serde(default)]
    pub noul: Option<f64>,
    #[serde(default)]
    pub choice: Option<String>,
    #[serde(default)]
    pub score: Option<f64>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub probabilities: Option<BTreeMap<String, f64>>,
}

/// System One wire response payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemOneWireResponse {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub answers: BTreeMap<String, SystemOneWireAnswer>,
    #[serde(default)]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}
```

## 3. Wire Translation Helpers
To avoid duplicating the question mapping and answer parsing between `jev.rs` and `kev.rs`, `types.rs` exposes:
- `build_systemone_questions(questions: Vec<DecisionQuestion>) -> BTreeMap<String, SystemOneWireQuestion>`: Maps `DecisionQuestion::Boolean` to `noul`, `DecisionQuestion::Choice` to `choice`, `DecisionQuestion::Score` to `score`.
- `parse_systemone_answers(answers: BTreeMap<String, SystemOneWireAnswer>) -> BTreeMap<String, DecisionAnswer>`: Parses `choice`, `noul` (bool >= 0.5), and `score` with confidence and probabilities.

## 4. JevProvider Implementation (`src/decisions/jev.rs`)
- URL construction:
  ```rust
  let trimmed = self.config.endpoint.trim_end_matches('/');
  let url = if trimmed.ends_with("/v1") {
      format!("{trimmed}/systemone")
  } else {
      format!("{trimmed}/v1/systemone")
  };
  ```
- Request building:
  Builds `state` with `task_description`, `workflow_stage`, `execution_mode`, and `metadata`. Sets `model` from `self.config.model`. Uses `build_systemone_questions`.
- Response parsing:
  Parses `SystemOneWireResponse`. Extracts cost from `usage["estimated_cost_usd"]` if present. Uses `parse_systemone_answers`.

## 5. Companion Tools Index Detection (`src/commands/tools.rs`)
Update `init_codegraph`:
```rust
let codegraph_dir = target_path.join(".codegraph");
let db_file = codegraph_dir.join("codegraph.db");
if codegraph_dir.exists() && db_file.exists() {
    if !ctx.quiet {
        println!(
            "tools: codegraph index (.codegraph/) is already initialized at '{}'",
            target_path.display()
        );
    }
    return Ok(());
}
```
If `.codegraph` exists but `codegraph.db` is missing, it will proceed to invoke `codegraph init` rather than prematurely returning.
