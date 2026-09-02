# Execution Plan: Session-Start Drift Delivery & Hook Architecture

**Date:** 2026-09-02  
**Status:** In Execution  
**OpenSpec Change:** `openspec/changes/session-start-drift-delivery/`  
**Governing Skill:** `/ce-plan`

---

## 1. Context & Objective
Bridge the Turn-0 delivery gap for `ce-ai workflow resume`:
1. Native shell lifecycle hook (`SessionStart`) for Claude Code in `.claude/settings.json`.
2. Explicit Turn-0 prompt directive in `AGENTS.md` managed block with `BLOCK_VERSION: 4`.
3. Checkpoint & doctor health audit gates.
4. Accurate documentation removing outdated placeholder caveats and overpromises.

---

## 2. Work Units & Execution Order

1. **Work Unit 1: Claude Code Settings Hook Management (`src/harness/claude.rs`)**
   - Implement `ensure_session_start_hook`, `remove_session_start_hook`, `has_session_start_hook`.
   - Unit tests covering JSON preservation, idempotency, and surgical removal.
2. **Work Unit 2: Adoption Engine Integration & Block Bump v4 (`src/commands/init_prj.rs` & `deinit_prj.rs`)**
   - Inject Turn-0 directive in `render_block_content(AdoptionTier::Full)`.
   - Bump `BLOCK_VERSION: 4`.
   - Hook into `init_prj` and `deinit_prj`.
   - Coordinate `BLOCK_VERSION` fixtures in `tests/cli.rs`.
3. **Work Unit 3: Checkpoint & Doctor Health Gates (`src/commands/workflow.rs` & `doctor.rs`)**
   - Drift warnings in `workflow::checkpoint_lines`.
   - `claude-hook-missing` check in `doctor::run`.
   - Integration tests in `tests/cli.rs`.
4. **Work Unit 4: Documentation Alignment**
   - Update `zero-step-drift-recovery-explained.md`.
   - Update `harnesses-loops-and-context-masterclass.md`.
   - Update `workflow-panel-native-vs-agent-skills.md`.

---

## 3. Verification & Quality Gates
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (100% pass)
- `make e2e` (containerized Docker gate)
