# Proposal: `exit-codes-and-benches-wiring`

## Why
Issues #163 (P1) and #84.

- #163: invariant #7 promises exit codes `3` State / `4` IO / `5` Network,
  but `CeError` folds them into Runtime(1)/Io(1) — automation cannot pick
  remediation paths. Verification(6) was already added in v1.18.1.
- #84: `benches/benchmarks.rs` holds stable-`#[test]` perf bounds (<50ms)
  that no target ever runs, and pins a stale `0.9.0` version literal.

## What Changes
- Add `CeError::State(String)` → exit 3 and `CeError::Network(String)` →
  exit 5; remap `Io` → exit 4.
- Construct `State(3)` from state.json load/parse/persist failures;
  construct `Network(5)` from propagating GitHub tarball transport failures.
- Makefile `bench` target (`cargo test --benches --release`); benches pin
  version via `env!("CARGO_PKG_VERSION")`.
