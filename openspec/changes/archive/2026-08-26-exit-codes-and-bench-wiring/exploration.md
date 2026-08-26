# Exploration
- resolve_latest_release network errors intentionally fall back to the main
  tarball with a notice (SF-2 resilience, v1.18.0) — those are NOT error
  paths; only propagating transports become Network(5).
- State wrapping stays inside State::load/save so all ~30 call sites inherit
  semantics without signature churn.
- Benches are plain #[test] fns: `cargo test --benches --release` runs them
  on stable; release profile keeps the 50ms assertions meaningful.
