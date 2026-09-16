# Tasks: Adaptive Mode Router & Graduation Bridge

- [x] **Work Unit 1: Domain Models & State Serialization (`ExecutionMode`)** (~120 LOC)
  - [x] Define `ExecutionMode` enum (`Auto`, `Organic`, `Compound`) in `src/state/state.rs` with serde rename attributes and `Default`.
  - [x] Implement `ExecutionMode::parse(s: &str) -> Result<Self, CeError>` supporting `odd`, `organic`, `compound`, `ce`, `auto`.
  - [x] Add `pub execution_mode: Option<ExecutionMode>` to `WorkflowState` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  - [x] Add unit tests in `src/state/tests/state.rs` for `ExecutionMode::parse` and `WorkflowState` serialization round-trips.
  - [x] Verification: `cargo test state::tests::state`.

- [x] **Work Unit 2: Deterministic Turn-0 Mode Classification & Banner Delivery** (~180 LOC)
  - [x] Implement `probe_execution_mode(repo_root: &Path, branch: Option<&str>, wf: &Option<WorkflowState>, tier: AdoptionTier, cli_override: Option<ExecutionMode>) -> ExecutionMode` in `src/commands/workflow.rs`.
  - [x] Wire CLI `--mode` override flag into `Action::Resume`, `Action::Status`, and `Action::Checkpoint`.
  - [x] Implement OpenSpec precedence rule (active `openspec/changes/<feature>/` forces Compound mode).
  - [x] Implement branch prefix classification (`fix/`, `chore/`, `spike/`, `test/` ➔ Organic; `feat/`, `spec/` ➔ Compound).
  - [x] Implement non-git directory fallback (`odd/tasks/` vs `openspec/changes/`, and `AdoptionTier`).
  - [x] Customize `resume_lines()`: emit ODD guidance block in Organic mode, suppress OpenSpec warnings.
  - [x] Add unit tests in `src/commands/tests/workflow.rs` for `probe_execution_mode` heuristics and banner rendering.
  - [x] Verification: `cargo test commands::tests::workflow`.

- [x] **Work Unit 3: Canonical ODD Brief Schema & Generation/Parsing Helpers** (~110 LOC)
  - [x] Implement `generate_odd_task_template(feature: &str) -> String` producing canonical markdown brief.
  - [x] Implement `OddTaskContent`, `OddDoDItem` structs, and parser `parse_odd_task_file(path: &Path) -> Result<OddTaskContent, CeError>`.
  - [x] Add unit tests in `src/commands/tests/workflow.rs` for template generation and lossless parsing of checkboxes (`[x]` and `[ ]`).
  - [x] Verification: `cargo test commands::tests::workflow`.

- [x] **Work Unit 4: Graduation Bridge Subcommand (`workflow graduate`)** (~190 LOC)
  - [x] Define `GraduateArgs` struct in `src/commands/workflow.rs` and add `Action::Graduate(GraduateArgs)`.
  - [x] Implement `run_graduate(ctx: &Context, args: &GraduateArgs) -> Result<(), CeError>`:
    - [x] Validate source file `odd/tasks/<feature>.md` exists (fail with `CeError::Usage` exit code 2 if missing).
    - [x] Validate destination `openspec/changes/<feature>` does not exist (fail with `CeError::State` exit code 3 if collision).
    - [x] Parse ODD task file and write `proposal.md`, `spec.md`, and `tasks.md` with preserved checkboxes.
    - [x] Remove `odd/tasks/<feature>.md` upon successful write.
    - [x] Atomically update `state.json` transitioning directly to Stage 4 (`WorkTdd`) in `Compound` mode.
  - [x] Add unit tests in `src/commands/tests/workflow.rs` for graduation error conditions and successful transformation.
  - [x] Verification: `cargo test commands::tests::workflow`.

- [x] **Work Unit 5: Dual-Track Gate Engine with Tactical LOC Ceilings** (~140 LOC)
  - [x] Update `evaluate_gate_policy` in `src/commands/gate.rs`:
    - [x] Allow tool writes without OpenSpec package when `ExecutionMode::Organic` is active.
    - [x] Inspect git diff line count in Organic mode: if > 200 LOC, emit observe-only advisory notice recommending graduation.
    - [x] Ensure exit code remains 0 (`GateDecision::Pass`).
  - [x] Add unit tests in `src/commands/tests/gate.rs` testing Organic write pass, diff < 200 LOC clean pass, and diff > 200 LOC advisory pass.
  - [x] Verification: `cargo test commands::tests::gate`.

- [x] **Work Unit 6: CLI Registry, End-to-End Integration Tests & Verification** (~140 LOC)
  - [x] Register top-level alias `ce-ai graduate` in `src/commands/registry.rs` and CLI entry point in `src/main.rs`.
  - [x] Add CLI integration test in `tests/cli.rs` covering the full end-to-end lifecycle: Organic task creation ➔ `workflow resume` banner ➔ `gate check` pass ➔ `workflow graduate` promotion ➔ verification of OpenSpec artifacts.
  - [x] Run complete verification suite:
    - [x] `cargo fmt --check`
    - [x] `cargo clippy --all-targets --all-features -- -D warnings`
    - [x] `cargo test`
  - [x] Verification: `cargo test`.
