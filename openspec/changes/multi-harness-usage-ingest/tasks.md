# Tasks: Multi-Harness Usage Ingestion Adapters

- [ ] **Unit 1 (~120 LOC):** `UsageAdapter` trait & Claude refactoring
  - Define `UsageAdapter` trait in `src/harness/usage/mod.rs`.
  - Refactor `src/harness/usage/claude.rs` to implement `UsageAdapter`.
  - Provide an adapter registry iterator in `src/harness/usage/mod.rs`.
  - *Verification*: `cargo test --test usage` confirms existing Claude usage ingestion tests pass.

- [ ] **Unit 2 (~160 LOC):** OpenCode usage ingestion adapter & fixtures
  - Create realistic test fixture in `tests/fixtures/usage/opencode/session.json`.
  - Implement `src/harness/usage/opencode.rs` parsing OpenCode session JSON events.
  - Handle missing token fields gracefully without creating artificial zero-token records.
  - *Verification*: Unit tests in `src/harness/usage/opencode.rs` verify extraction from fixtures and edge-case handling.

- [ ] **Unit 3 (~180 LOC):** Codex and Pi usage ingestion adapters & fixtures
  - Create test fixtures for Codex (`tests/fixtures/usage/codex/session.jsonl`) and Pi (`tests/fixtures/usage/pi/session.json`).
  - Implement `src/harness/usage/codex.rs` mapping OpenAI tokens, cache, and reasoning breakdown.
  - Implement `src/harness/usage/pi.rs` mapping Pi agent session turn statistics.
  - *Verification*: Unit tests verify accurate token mappings from static fixtures.

- [ ] **Unit 4 (~130 LOC):** `ce-ai usage sync` multi-harness dispatch & CLI flag
  - Add optional `--harness <name|all>` argument to `UsageCommand::Sync` in `src/commands/usage.rs`.
  - Iterate through active adapters, query `is_available`, ingest records, and output per-harness summaries.
  - Ensure deduplication idempotency via `dedup_key`.
  - *Verification*: Integration tests in `tests/commands/usage.rs` covering multi-harness sync, scoped sync, and invalid harness rejection.

- [ ] **Unit 5 (~60 LOC):** Documentation, CLI help strings & release preparation
  - Update `ce-ai usage sync --help` descriptions in `src/commands/usage.rs`.
  - Update `docs/user-guide/harness-matrix.md` and `CHANGELOG.md` with multi-harness usage support.
  - *Verification*: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`, and `ce-ai doc lint --strict`.
