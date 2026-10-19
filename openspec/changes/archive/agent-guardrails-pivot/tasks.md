# Tasks: Agent Guardrails & Risk Engine Pivot (`ce-ai guard`)

## Work Units & Changed-Line Estimates

- [x] **Unit 1: Implement `src/commands/guard.rs` and CLI Command Dispatch** (~190 LOC)
  - [x] Define `guard` action enum (`Check`, `Status`, `Setup`, `Policy`, `Test`, `Enable`, `Disable`) in `src/commands/guard.rs`.
  - [x] Register `Commands::Guard` in `src/commands/registry.rs` and dispatch in `src/main.rs`.
  - [x] Implement `handle_check` with deterministic safety evaluation, JSON serialization, and latency measurement.
  - [x] Implement backward-compatible alias from `ce-ai decisions check-risk` to `ce-ai guard check`.

- [x] **Unit 2: Deprecate Model Routing and Dead Comparison Code** (~180 LOC)
  - [x] Update `handle_route` in `src/commands/decisions.rs` to emit a deprecation advisory and exit with code 0.
  - [x] Update `handle_compare` in `src/commands/decisions.rs` to emit a deprecation advisory and exit with code 0.
  - [x] Cleanly prune dead model routing catalog dependencies and comparison algorithms in `src/decisions/`.
  - [x] Update decision routing tests to verify deprecation notices and exit code 0 behavior.

- [x] **Unit 3: Re-align Presets & Doctor Probes to Local-First Safety** (~190 LOC)
  - [x] Update setup presets in `src/commands/guard.rs` and `src/commands/decisions.rs`:
    - Auto-detect Apple Silicon macOS (`aarch64`) to configure `laya` in `recommended` preset.
    - Default universal platforms to local deterministic safety rules without prompting for or requiring `TYPESAFE_API_KEY`.
    - Retain `jev` purely as an explicit opt-in provider (`--provider jev`).
  - [x] Update `src/commands/doctor.rs`:
    - Add `doctor-info: guard: active (...)` diagnostic reporting.
    - Eliminate unauthenticated false-positive warnings for Jev when the user did not explicitly configure TypeSafe AI.

- [x] **Unit 4: Guardrail Unit and Integration Test Suite** (~200 LOC)
  - [x] Create `src/commands/tests/guard.rs` covering:
    - Deterministic `Deny` for catastrophic commands (`rm -rf /`, `git push -f origin main`, `.ssh` read) in < 5 ms.
    - `Allow` for benign development commands (`cargo test`, `git status`, `npm run build`).
    - JSON output schema conformance (`--json`).
    - Backward-compatible alias execution via `ce-ai decisions check-risk`.
    - Deprecation notices for `ce-ai decisions route` and `ce-ai decisions compare`.

- [x] **Unit 5: Governance, Documentation & Concept Accretion** (~120 LOC)
  - [x] Monotonically accrete domain concepts in `CONCEPTS.md` (`Agent Guardrails Engine`, `Deterministic Safety Precedence`).
  - [x] Verify concept integrity with `cargo run -- doc lint --strict`.
  - [x] Document architectural solution in `docs/solutions/architecture/agent-guardrails-and-risk-engine-pivot.md`.
  - [x] Update `docs/user-guide/decision-engine-guide.md` and `README.md`.
  - [x] Bump version in `Cargo.toml` and update `CHANGELOG.md`.

- [x] **Unit 6: Verification & Quality Gates** (0 LOC)
  - [x] Verify formatting: `cargo fmt --check`.
  - [x] Verify linter: `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Verify unit & integration test suite: `cargo test`.
  - [x] Verify containerized E2E gate: `make e2e`.
