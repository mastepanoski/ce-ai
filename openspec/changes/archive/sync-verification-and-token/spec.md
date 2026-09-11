# Spec: Fix `ce-ai upgrade` Verification Drift Error and GitHub Token Discovery

## 1. Requirements

### Requirement 1: Multi-Tiered GitHub Token Discovery
- **WHEN** `github_token_from_env()` is invoked,
- **THEN** it MUST inspect environment variables in priority order:
  1. `CE_AI_GITHUB_TOKEN`
  2. `GITHUB_TOKEN`
  3. `GH_TOKEN`
- **WHEN** none of the environment variables are set or non-empty,
- **THEN** it MUST attempt to query `gh auth token` via `std::process::Command`.
- **WHEN** `gh auth token` returns exit code 0 and non-empty output,
- **THEN** that trimmed token is returned.
- **WHEN** all resolution attempts fail,
- **THEN** `None` is returned without error or panic.

### Requirement 2: Native Harness Verification Matrix Classification
- **WHEN** `sync_with()` executes verification matrix checks,
- **AND** an active harness is a table-driven native harness (`claude`, `codex`, `copilot`, `grok`, `kimi`, `agy`, `pi`, `fx`),
- **AND** the harness is not recorded as `adopted` or `orphaned` in `state.skill_surfaces`,
- **AND** the harness root is not detected as `pending_adoptions`,
- **THEN** the harness MUST be classified as:
  ```rust
  CheckStatus::NotVerified {
      reason: REASON_NO_MANAGED_SKILLS,
  }
  ```
- **THEN** the reconciliation summary MUST report:
  ```
  reconciliation status: 1 verified, N registered (nothing to verify), 0 failed
  ```
- **THEN** `sync_with()` MUST succeed with exit code 0 rather than failing with `CeError::Verification`.

## 2. Acceptance Criteria
1. `cargo test` passes 100% of unit and integration tests.
2. An integration test in `tests/cli.rs` confirms that with `.claude` present on the host filesystem, `ce-ai sync` and `ce-ai upgrade` succeed without drift errors and print `registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)`.
3. An integration test confirms that adopted surfaces in `state.skill_surfaces` remain strictly verified and report real drift when modified.
