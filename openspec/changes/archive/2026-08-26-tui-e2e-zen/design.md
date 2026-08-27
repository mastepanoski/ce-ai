# Design: tui-e2e-zen

## Architecture
- `src/tui.rs` remains thin delegator to `capture_cli`; contract test is source of truth.
- `TestBackend(80,24)` renders `ui()` without `enable_raw_mode` or `is_terminal`.
- `e2e_runner.sh` reuses existing isolated `HOME=/tmp/ce-ai-home`; new steps run after E2E 9.

## Sequence
1. `cargo test tui` → `TestBackend` snapshots (T2)
2. `cargo test` → 15-vector contract (T1)
3. `make e2e` → docker build → inside container: `ce-ai skills resolve` + `tools status` + `cargo test tui` (T3)

## Data
- No state schema change; only test + E2E assets.
