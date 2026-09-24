# Proposal: Remediate Hallucinated Jev Endpoint to TypeSafe System One Wire Protocol

## 1. Problem Statement
During an audit of the decision engine codebase, `src/decisions/jev.rs` was identified as having a hallucinated wire protocol and endpoint:
- `JevProvider::evaluate` calls `format!("{}/decide", endpoint)` with an unauthenticated or custom request format `JevWireRequest { questions: Vec<DecisionQuestion>, ... }`.
- In reality, TypeSafe AI's API exposes `POST /v1/systemone` using the standard System One wire protocol (`state`, `model`, `questions: BTreeMap<String, SystemOneWireQuestion>` with `noul`, `choice`, `score` primitives).
- Calling `/decide` on `api.typesafe.ai` yields HTTP 404 Not Found.
- Conversely, `src/decisions/kev.rs` and `src/decisions/laya.rs` already implement the correct System One wire protocol, but their definitions are private/local to `kev.rs`.
- In addition, `ce-ai tools init codegraph` considered `.codegraph/` already initialized if only `.codegraph/.gitignore` was checked out from git, rather than verifying `codegraph.db`.

## 2. In-Scope Boundaries
- Move or centralize the shared System One wire structures (`SystemOneWireQuestion`, `SystemOneWireRequest`, `SystemOneWireAnswer`, `SystemOneWireResponse`) into `src/decisions/types.rs` (with backward-compatible re-exports in `src/decisions/kev.rs`).
- Refactor `src/decisions/jev.rs` to build and transmit `SystemOneWireRequest` targeting `POST /v1/systemone` (or `{endpoint}/systemone` if endpoint already ends with `/v1`).
- Parse the System One response into `DecisionResponse` with proper mapping of `noul` (boolean), `choice` (categorical), and `score` (numeric) primitives, including confidence, probabilities, and estimated cost if present.
- Update tests in `src/decisions/tests/jev_tests.rs` to validate the real System One wire payload serialization and response deserialization.
- Improve `src/commands/tools.rs` so `ce-ai tools init codegraph` checks for `codegraph.db` rather than merely the directory, avoiding false-positive "already initialized" reports in fresh worktrees.
- Verify through unit tests, live provider tests, and full CI quality gates.

## 3. Out-of-Scope Boundaries
- Modifying Kev or Laya provider evaluation logic or changing their external behavior.
- Altering the domain `DecisionRequest` or `DecisionResponse` public APIs used by `ce-ai` workflows.
- Modifying unrelated code in the main checkout or touching `docs/reposition-ce-ai`.

## 4. Risk Evaluation
- **Wire Payload Compatibility**: The System One wire protocol is already validated and running in `kev.rs` and `laya.rs`. Reusing the identical schema minimizes risk.
- **Provider Parity**: With `jev.rs` aligned to the same schema as `kev` and `laya`, all three providers become cross-compatible drop-in alternatives.
- **Regression Risk**: Low. `jev.rs` was previously broken against live TypeSafe AI endpoints due to `/decide` 404; aligning with `/systemone` restores real functionality.

## 5. Success Criteria
- `JevProvider::evaluate` targets `/v1/systemone` with a valid `SystemOneWireRequest`.
- Wire serialization and deserialization tests pass with 100% coverage in `src/decisions/tests/jev_tests.rs`.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` pass with 0 errors.
- `make e2e` passes.
- Version bumped to `1.68.2` and documented in `CHANGELOG.md`.
