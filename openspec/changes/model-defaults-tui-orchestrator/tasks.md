# Tasks: model-defaults-tui-orchestrator

## 1. Defaults + orchestrator slot (models.rs, install.rs)
- [x] 1.1 RED: tests — `apply_defaults` seeds `ce-ai` + stage slots on empty config; records in state.json.
- [x] 1.2 GREEN: add `ORCHESTRATOR_SLOT`, `DEFAULT_MODEL_ASSIGNMENTS`, `apply_defaults`; widen `set` to `pub(crate)`.
- [x] 1.3 Wire `apply_defaults` into `install.rs` (post-save, non-dry-run); print seeded pairs.
- [x] 1.4 RED→GREEN: test — existing user model in config/state is never overwritten.

## 2. Harness-driven model discovery (models.rs)
- [x] 2.1 RED: test — `parse_models_output` extracts `provider/model` tokens, drops annotations and malformed lines.
- [x] 2.2 GREEN: implement parser + `discover_models()` (hard-fail on CLI absence/exit/empty — no static catalog).

## 3. Drift detection (models.rs, doctor.rs)
- [x] 3.1 RED: tests for `model_drift_findings` (divergent, state-only, untracked CE slot, third-party ignored).
- [x] 3.2 GREEN: implement helper; wire into doctor findings.

## 4. Sync import (sync.rs)
- [x] 4.1 RED: tests for `import_config_assignments` (missing→imported, divergent→updated, equal→noop, malformed skipped).
- [x] 4.2 GREEN: implement helper + wire into `sync_with` before final `state.save`; print imports.

## 5. TUI Models tab editing (tui.rs)
- [x] 5.1 Add App picker state + editable slot list (defaults ∪ tracked); slot cursor rendering with hints.
- [x] 5.2 Key handling: picker nav/confirm/cancel (`n`/`p`, `m`, Up/Down, Enter, Esc).
- [x] 5.3 Confirm path calls `models::set`; picker populated via `discover_models()`; errors surface in modal.
- [x] 5.4 Render picker modal with selection highlight.

## 6. Verification
- [x] 6.1 `cargo fmt --check`
- [x] 6.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 6.3 `cargo test`
- [x] 6.4 CHANGELOG.md entry (Keep a Changelog, Unreleased).
- [x] 6.5 `make e2e` (containerized Docker gate).
