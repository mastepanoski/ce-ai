# Specification: Decision Engine Documentation & CLI Ergonomics

## Requirements

### R1: Setup Preset `off` / `disabled`
- **WHEN** executing `ce-ai decisions setup --preset off` (or `--preset disabled`)
- **THEN** `state.decisions` MUST be updated with `enabled = false` and `mode = DecisionMode::Off`.
- **THEN** the update MUST be persisted to disk via `crate::state::write_atomic`.
- **THEN** the command MUST exit with code 0 and display confirmation that the engine is disabled.
- **WHEN** an invalid preset is provided
- **THEN** the error message MUST list `recommended`, `shadow`, `local`, `off` as the valid options.

### R2: Operational Mode Subcommand (`ce-ai decisions mode`)
- **WHEN** executing `ce-ai decisions mode` without arguments
- **THEN** it MUST output the current operational mode (`active`, `shadow`, or `off`) and provider.
- **WHEN** executing `ce-ai decisions mode <mode>` with a valid mode (`active`, `shadow`, `off`, `disabled`, `on`)
- **THEN** it MUST parse the target mode using `DecisionMode::parse`.
- **THEN** it MUST atomically persist the updated mode to `state.json`.
- **THEN** setting mode to `off` MUST mark `enabled = false`.
- **THEN** setting mode to `active` or `shadow` MUST mark `enabled = true`.
- **WHEN** executing `ce-ai decisions mode <invalid>`
- **THEN** it MUST return `CeError::Usage` with exit code 2 and list the valid modes (`off`, `shadow`, `active`).

### R3: Comprehensive User Guide Documentation
- **WHEN** navigating to `docs/user-guide/decision-engine-guide.md`
- **THEN** it MUST follow Diátaxis How-to / Reference standards without blending conflicting quadrants.
- **THEN** it MUST document:
  1. The distinction between Presets (`recommended`, `shadow`, `local`, `off`) and Modes (`active`, `shadow`, `off`).
  2. Configuration and authentication (`ce-ai decisions auth`, `ce-ai decisions status`).
  3. Risk evaluation (`ce-ai decisions check-risk`).
  4. Readiness verification (`ce-ai decisions check-readiness`).
  5. Model and skill routing (`ce-ai decisions route`).
  6. Deterministic safety invariants and circuit breaker fallbacks.

### R4: README Line Count & Navigation Contract
- **WHEN** updating `README.md`
- **THEN** the total line count of `README.md` MUST NOT exceed 100 lines.
- **THEN** the Command Table MUST link to `docs/user-guide/decision-engine-guide.md`.
- **THEN** the Documentation Map MUST contain an entry for `Decision Engine Guide` with audience labeling.

---

## Acceptance Criteria

1. `cargo test` passes 100% including new CLI integration tests for `ce-ai decisions setup --preset off` and `ce-ai decisions mode`.
2. `ce-ai decisions setup --preset off` results in `status` reporting disabled with exit code 0.
3. `ce-ai decisions mode active` switches disabled engine to active and `ce-ai decisions mode off` cleanly disables it.
4. `wc -l README.md` outputs `<= 100`.
5. Zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
