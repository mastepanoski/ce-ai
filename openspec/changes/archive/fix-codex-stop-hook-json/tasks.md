# Tasks: Codex and Multi-Harness Stop Hook Clean JSON Output

- [x] **Work Unit 1: Add `--event` Flag & Stdin Hook Detection in `workflow resume`** (~80 LOC)
  - [x] Add `#[arg(long, value_name = "EVENT")] event: Option<String>` to `Action::Resume` in `src/commands/workflow.rs`.
  - [x] Define `HookPayload` and helper `resolve_hook_context` in `src/commands/workflow.rs`.
  - [x] In `Action::Resume` handler, intercept `Stop` and `PreCompact` events:
    - [x] If `stop_hook_active == Some(true)`, emit `{}` and return `Ok(())`.
    - [x] Run `maybe_auto_checkpoint(ctx, &repo_root, &state_path)`.
    - [x] Emit `{}` and return `Ok(())`.
  - [x] For `SessionStart` / default events: retain JSON payload or text lines based on `--json` and hook invocation mode.
  - [x] Verification: `cargo check`.

- [x] **Work Unit 2: Update Harness Detectors in `codex.rs` and `claude.rs`** (~40 LOC)
  - [x] Update `has_codex_event_hook` in `src/harness/codex.rs` to match exact command or prefix `ce-ai workflow resume`.
  - [x] Update `remove_session_start_hook` in `src/harness/codex.rs` to match exact command or prefix `ce-ai workflow resume`.
  - [x] Update `has_event_hook` and `remove_session_start_hook` in `src/harness/claude.rs` to match exact command or prefix `ce-ai workflow resume`.
  - [x] Verification: `cargo test --lib harness`.

- [x] **Work Unit 3: Add Unit & Integration Tests for Stop/PreCompact Hooks** (~100 LOC)
  - [x] Add tests in `src/commands/tests/workflow.rs` verifying:
    - [x] `workflow resume --event Stop` emits `{}` and exits 0.
    - [x] `workflow resume --event PreCompact` emits `{}` and exits 0.
    - [x] Non-terminal `stdin` with `{"hook_event_name": "Stop"}` emits `{}` and exits 0.
    - [x] Non-terminal `stdin` with `{"stop_hook_active": true}` emits `{}` and exits 0 without looping.
    - [x] Interactive terminal simulation preserves human text lines.
  - [x] Verification: `cargo test`.

- [x] **Work Unit 4: Quality Gates, Versioning & CHANGELOG** (~25 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Run `make e2e`.
  - [x] Bump patch version in `Cargo.toml` (`1.63.1`).
  - [x] Update `CHANGELOG.md`.
  - [x] Verification: `cargo test`.
