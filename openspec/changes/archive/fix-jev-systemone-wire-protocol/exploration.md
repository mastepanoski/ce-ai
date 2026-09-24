# Exploration: Jev Wire Protocol Alignment with TypeSafe System One

## 1. Technical Investigation
Auditing `src/decisions/` revealed an inconsistency across providers:
- `src/decisions/kev.rs`: Targets `{base_url}/systemone`, serializes to `SystemOneWireRequest`, maps `DecisionQuestion` variants into `noul`, `choice`, and `score` questions.
- `src/decisions/laya.rs`: Also targets `{base_url}/systemone`, reuses `SystemOneWireQuestion` and `SystemOneWireResponse` from `crate::decisions::kev`.
- `src/decisions/jev.rs`: The cloud TypeSafe AI provider was implemented using a non-existent `/decide` endpoint and custom structs `JevWireRequest` and `JevWireResponse`. Testing against the live API `https://api.typesafe.ai` returns HTTP 404 for `/decide`.

Furthermore, inspecting the TypeSafe AI System One specification confirms:
- Endpoint: `POST https://api.typesafe.ai/v1/systemone`
- Request Schema:
  ```json
  {
    "state": {
      "task_description": "...",
      "workflow_stage": "...",
      "execution_mode": "...",
      "metadata": { ... }
    },
    "model": "jev-latest",
    "questions": {
      "question_id": {
        "type": "noul" | "choice" | "score",
        "instructions": "...",
        "criteria": { ... } | [ ... ]
      }
    }
  }
  ```
- Response Schema:
  ```json
  {
    "model": "jev-latest",
    "answers": {
      "question_id": {
        "type": "noul" | "choice" | "score",
        "noul": 0.94,
        "choice": "option_a",
        "score": 2.5,
        "confidence": 0.88,
        "probabilities": { "option_a": 0.88, "option_b": 0.12 }
      }
    },
    "latency_ms": 42,
    "usage": { "estimated_cost_usd": 0.0003 }
  }
  ```

## 2. Companion Tool Initialization Investigation
When running `ce-ai tools init codegraph .`, the command reported:
`tools: codegraph index (.codegraph/) is already initialized at '.'`
However, `.codegraph` only contained `.gitignore` which was committed to git in the root repo. The actual index file `codegraph.db` was not present, leading to failure when running `codegraph_explore`.
Checking `src/commands/tools.rs:210`:
```rust
let codegraph_dir = target_path.join(".codegraph");
if codegraph_dir.exists() {
    ...
    return Ok(());
}
```
Checking only directory existence causes false-positive skips when `.codegraph` contains only git metadata. Checking `target_path.join(".codegraph").join("codegraph.db").exists()` ensures that an unindexed checkout is properly initialized.

## 3. Evaluated Options for Wire Struct Architecture
- **Option A (Duplication)**: Define `JevWireQuestion`, `JevWireRequest`, etc. in `jev.rs` independently.
  - *Trade-off*: Duplicates ~100 lines of identical serialization code across `kev.rs` and `jev.rs`. Violates DRY and increases maintenance burden.
- **Option B (Import from kev.rs)**: Have `jev.rs` import `SystemOneWireRequest` from `kev.rs` (as `laya.rs` currently does).
  - *Trade-off*: Inverted dependency. Kev is a local provider; having the primary cloud provider depend on Kev is conceptually backward.
- **Option C (Shared Types in `src/decisions/types.rs`) [CHOSEN]**: Move the canonical `SystemOneWireQuestion`, `SystemOneWireRequest`, `SystemOneWireAnswer`, and `SystemOneWireResponse` to `src/decisions/types.rs`. Re-export them in `kev.rs` so all existing call sites and external references remain unbroken.
  - *Rationale*: Clean domain separation. All providers (`jev`, `kev`, `laya`) share the official System One wire protocol structs from `types.rs`.
