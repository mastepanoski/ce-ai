# Tasks: Local Decision Engine Providers (Kev & Laya-MLX)

## Work Units

### Work Unit 1: State Configuration & Types (~120 LOC)
- [x] Extend `DecisionsConfig` in `src/state/state.rs` with `KevConfig` and `LayaConfig`.
- [x] Add default constructors and serde serialization/deserialization tests.
- [x] Ensure backward compatibility with existing `state.json` without clobbering unmanaged keys.
- [x] TDD: Unit test `KevConfig` and `LayaConfig` serialization roundtrips.

### Work Unit 2: Kev Provider Implementation (`src/decisions/kev.rs`) (~180 LOC)
- [x] Implement `KevProvider` conforming to the `DecisionProvider` trait.
- [x] Implement System One wire serialization (`build_wire_payload`) supporting `noul`, `choice`, `score`.
- [x] Implement response deserialization (`parse_wire_response`) handling choice probabilities and confidence.
- [x] Implement `check_health` probing `GET /v1/models` (or `/health`).
- [x] TDD: Unit test wire serialization with mock payloads.

### Work Unit 3: Laya-MLX Provider Implementation (`src/decisions/laya.rs`) (~170 LOC)
- [x] Implement `LayaMlxProvider` conforming to the `DecisionProvider` trait.
- [x] Implement HTTP/UDS transport client with short default timeout (250ms).
- [x] Map domain `DecisionRequest` to Laya encoder decision head payloads.
- [x] Implement `check_health` checking daemon/socket availability.
- [x] TDD: Unit test Laya provider serialization and error handling.

### Work Unit 4: Factory Integration & CLI Setup/Test (~190 LOC)
- [x] Wire `KevProvider` and `LayaMlxProvider` into `DecisionEngine::from_config` in `src/decisions/mod.rs`.
- [x] Update `ce-ai decisions setup` in `src/commands/decisions.rs` to support `--provider kev`, `--provider laya`, `--preset kev`, `--preset laya`.
- [x] Add `--endpoint` flag support to override default localhost addresses.
- [x] Implement `ce-ai decisions test` command in `src/commands/decisions.rs` to test active provider connectivity.
- [x] TDD: Unit test `DecisionEngine::from_config` with both provider configurations.

### Work Unit 5: Doctor Diagnostics & Cross-Platform Gate (~140 LOC)
- [x] Update `ce-ai doctor` in `src/commands/doctor.rs` to inspect active provider health.
- [x] If `kev`, probe endpoint and provide actionable start command if unreachable.
- [x] If `laya`, detect `aarch64-apple-darwin` platform support and probe socket/HTTP.
- [x] TDD: Unit test doctor probe logic with mocked servers.

### Work Unit 6: End-to-End Verification & Documentation (~120 LOC)
- [x] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
- [x] Update `docs/` and `README.md` documenting local Decision Engine setup (`kev` and `laya-mlx`).
- [x] Prepare changelog entry.

Total estimated changed lines: ~920 LOC across 6 atomic work units.
