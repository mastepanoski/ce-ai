# Specification: TypeSafe System One Wire Protocol for JevProvider

## 1. Requirements

### REQ-JEV-1: Canonical System One Wire Schema
- **WHEN** serializing or deserializing System One payloads for any provider (`jev`, `kev`, `laya`),
- **THEN** the system MUST use `SystemOneWireRequest`, `SystemOneWireResponse`, `SystemOneWireQuestion`, and `SystemOneWireAnswer` defined in `src/decisions/types.rs`.

### REQ-JEV-2: Jev Wire Request Construction
- **WHEN** `JevProvider::build_wire_payload` is called with a `DecisionRequest`,
- **THEN** it MUST produce a `SystemOneWireRequest` containing:
  - `model`: matching `jev` config model (default `jev-latest`).
  - `state`: an object containing `task_description`, `workflow_stage`, `execution_mode`, and `metadata`.
  - `questions`: a map of question IDs to `SystemOneWireQuestion` with `type: "noul"` for boolean, `type: "choice"` for categorical, and `type: "score"` for numeric ratings.

### REQ-JEV-3: Jev System One HTTP Endpoint
- **WHEN** `JevProvider::evaluate` sends an HTTP request,
- **THEN** it MUST target `{endpoint}/systemone` (or `{endpoint}/v1/systemone` if endpoint does not contain `/v1`),
- **AND** it MUST include `Authorization: Bearer <API_KEY>` and `Content-Type: application/json`.

### REQ-JEV-4: Jev Wire Response Deserialization
- **WHEN** `JevProvider::parse_wire_response` processes a `SystemOneWireResponse`,
- **THEN** it MUST translate:
  - `noul` values (0.0 to 1.0) into `DecisionAnswer::Boolean { value: noul >= 0.5, confidence }`.
  - `choice` values into `DecisionAnswer::Choice { selected, confidence, probabilities }`.
  - `score` values into `DecisionAnswer::Score { score, confidence }`.
  - `usage` or cost into `estimated_cost_usd` when present.

### REQ-JEV-5: Companion Tool Initialization Guard
- **WHEN** `ce-ai tools init codegraph [PATH]` is invoked,
- **THEN** it MUST check that both the `.codegraph/` directory and `.codegraph/codegraph.db` index file exist before concluding the workspace is already initialized.

## 2. Acceptance Criteria
1. `JevProvider` builds valid `SystemOneWireRequest` instances matching Kev and Laya.
2. `JevProvider::evaluate` calls `POST /v1/systemone` instead of the non-existent `/decide`.
3. Wire serialization and deserialization unit tests in `jev_tests.rs` pass.
4. `ce-ai tools init codegraph` correctly triggers `codegraph init` when `.codegraph/` is present without `codegraph.db`.
5. All test suites pass: `cargo test`.
6. Code quality gates pass: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`.
