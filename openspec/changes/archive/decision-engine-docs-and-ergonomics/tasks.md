# Tasks: Decision Engine Documentation & CLI Ergonomics

Total PR Forecast: ~380 LOC (Review boundary: <= 400 LOC)

## Work Unit 1: CLI Ergonomic Enhancements (`ce-ai decisions mode` & `--preset off`) (est. ~80 LOC)
- [x] Add `Action::Mode { mode: Option<String> }` to `src/commands/decisions.rs`.
- [x] Implement `handle_mode(ctx: &Context, mode: Option<&str>)` in `src/commands/decisions.rs` using `crate::state::write_atomic`.
- [x] Update `handle_setup` in `src/commands/decisions.rs` to support `"off" | "disabled"` preset.
- [x] Update error messaging in `handle_setup` to list `off` among valid presets.

## Work Unit 2: CLI Integration Tests (est. ~90 LOC)
- [x] Add integration test in `tests/cli.rs` verifying:
  - `ce-ai decisions setup --preset off` cleanly disables evaluations.
  - `ce-ai decisions mode` prints current mode.
  - `ce-ai decisions mode active` switches mode and enables engine.
  - `ce-ai decisions mode shadow` switches to shadow mode.
  - `ce-ai decisions mode off` disables engine.
  - `ce-ai decisions mode invalid_mode` returns `CeError::Usage` (exit code 2).

## Work Unit 3: User Guide & README Documentation (est. ~180 LOC)
- [x] Author `docs/user-guide/decision-engine-guide.md` covering:
  - System 1 concept, Presets vs Modes matrix.
  - Authentication, setup, and status inspection.
  - Risk checks, readiness advisory, and model routing.
  - Safety guarantees (deterministic overrides and circuit breaker).
- [x] Update `README.md` to link `decision-engine-guide.md` in Command Table and Documentation Map.
- [x] Verify `wc -l README.md` is strictly `<= 100`.

## Work Unit 4: Verification, Version Bump (1.65.0) & CHANGELOG (est. ~30 LOC)
- [x] Bump version from `1.64.0` to `1.65.0` in `Cargo.toml`.
- [x] Run `cargo check` and `cargo build` to refresh `Cargo.lock`.
- [x] Document changes under `[1.65.0] - 2026-09-21` in `CHANGELOG.md`.
- [x] Run complete verification suite:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
