# Tasks: Expose Mid-Tier Model Slot for ce-code-review

Work-unit changed-line estimates total: ~180 LOC target (~200 LOC target policy).

- [x] **Task 1: Harness agent slot definitions & predicates** (~25 LOC)
  - [x] 1.1 Add constants and predicates in `src/harness/agents.rs`: `CODE_REVIEW_MID_TIER_SLOT`, `CE_AGENT_STAGE_SLOTS`, `CE_AGENT_SLOTS` (7 items), `is_tier_slot`, `is_stage_slot`.
  - [x] 1.2 Unit tests in `src/harness/tests/agents.rs` validating slot inclusion and predicates.

- [x] **Task 2: Model list formatting and slot set support** (~60 LOC)
  - [x] 2.1 RED test in `src/commands/tests/models.rs`: `format_model_assignments` hierarchical rendering with mid-tier slot and fallback rendering when parent unset.
  - [x] 2.2 GREEN implementation in `src/commands/models.rs`: implement `format_model_assignments` and update `list()` to use it.
  - [x] 2.3 RED test in `src/commands/tests/models.rs`: `set()` with `ce-code-review-mid-tier`, verifying persistence in `opencode.json`, `state.json`, snapshot generation, and absence of `variant`.
  - [x] 2.4 Verify existing `set()` handles `ce-code-review-mid-tier` cleanly.

- [x] **Task 3: Doctor informational diagnostic note** (~65 LOC)
  - [x] 3.1 RED tests in `src/commands/tests/doctor.rs`: test `check_code_review_mid_tier_note` across full state matrix (configured without mid-tier, both configured, neither configured, empty config) and assert non-fatal behavior.
  - [x] 3.2 GREEN implementation in `src/commands/models.rs`: implement `check_code_review_mid_tier_note`.
  - [x] 3.3 Integration in `src/commands/doctor.rs`: call `check_code_review_mid_tier_note` and output `doctor-info:` without pushing to `findings`.
  - [x] 3.4 Integration test in `tests/cli.rs`: verify CLI output contains `doctor-info:` and exits 0.

- [x] **Task 4: Quality gates and verification** (~30 LOC)
  - [x] 4.1 Formatting check: `cargo fmt --check`.
  - [x] 4.2 Linter check: `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] 4.3 Test suite execution: `cargo test`.
  - [x] 4.4 Docker E2E gate: `make e2e`.

