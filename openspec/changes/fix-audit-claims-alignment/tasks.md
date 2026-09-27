# Tasks: Audit claims alignment

- [x] **Unit 1 (~70 LOC):** Add failing backup identity/discovery tests for Kimi, AGY, and FX; implement prefixed backup names. Verify `cargo test state::tests::backups`.
- [x] **Unit 2 (~100 LOC):** Add a hermetic Kimi install/uninstall regression test that preserves a colliding user MCP entry; keep AGY legacy cleanup independent of snapshot restoration. Verify targeted command tests and `cargo test`.
- [x] **Unit 3 (~90 LOC):** Correct resume help and public documentation boundaries for memory, FSM/gate enforcement, model backups, and the README anchor. Verify link target and `ce-ai doc lint --strict`.
- [x] **Unit 4 (~0 LOC):** Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e`.
