# Plan: Detect and Repair Legacy Gentle-AI Directives in ce-ai Doctor

## Overview
Detect and automatically repair stale `gentle-ai` residual directives (such as `<!-- gentle-ai:codegraph-guidance -->` and `gentle-ai codegraph init`) in global harness instruction files and project instruction files during `ce-ai doctor` and project adoption.

## Tasks

### Phase 1: Directive Scanning & Modernization (`src/commands/init_prj.rs`)
- [x] Add `PAT-GENTLE-AI` pattern to `scan_legacy_directives` in `src/commands/init_prj.rs` to detect `gentle-ai:codegraph-guidance`, `gentle-ai codegraph init`, and legacy `gentle-ai` sentinels.
- [x] Add surgical modernization/stripping for `<!-- gentle-ai:codegraph-guidance -->` in `migrate_legacy_directives`.
- [x] Add unit test in `src/commands/tests/init_prj.rs` verifying detection and removal of `gentle-ai` blocks.

### Phase 2: Global Harness Directive Probe (`src/commands/doctor.rs`)
- [x] Implement `probe_legacy_harness_directives` in `src/commands/doctor.rs` inspecting global instruction files (`~/.gemini/GEMINI.md`, `~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`, `~/.config/opencode/AGENTS.md`).
- [x] Wire the probe into `doctor::run` with support for `--fix` and `--strict` reporting.
- [x] Add unit test in `src/commands/tests/doctor.rs` verifying detection and auto-repair with `--fix`.

### Phase 3: Integration Verification & DoD
- [x] Run `cargo fmt --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test`.
- [x] Add CLI integration test in `tests/cli.rs`.

### Phase 4: Documentation & Versioning
- [x] Bump version in `Cargo.toml` (`3.10.0`) and run `cargo check` to update `Cargo.lock`.
- [x] Update `CHANGELOG.md` with new `doctor` probe and `init-prj` legacy migration capability.
