# Tasks: Session-Start Drift Delivery Implementation

## Work Unit 1: Claude Code Settings Hook Management (`src/harness/claude.rs`)
- [ ] Implement `ensure_session_start_hook(path: &Path) -> Result<bool, CeError>`. (~40 LOC)
- [ ] Implement `remove_session_start_hook(path: &Path) -> Result<bool, CeError>`. (~35 LOC)
- [ ] Implement `has_session_start_hook(path: &Path) -> bool`. (~25 LOC)
- [ ] Add unit tests in `src/harness/tests/claude.rs` for idempotency, preserving user keys, and surgical removal. (~60 LOC)

## Work Unit 2: Adoption Engine Integration & Block Bump v4 (`src/commands/init_prj.rs` & `deinit_prj.rs`)
- [ ] Update `render_block_content(AdoptionTier::Full)` with Turn-0 `ce-ai workflow resume` directive. (~15 LOC)
- [ ] Bump `BLOCK_VERSION: u32 = 4;` in `src/commands/init_prj.rs`. (~5 LOC)
- [ ] In `init_prj::run`, call `ensure_session_start_hook` when `.claude/` exists. (~20 LOC)
- [ ] In `deinit_prj::run`, call `remove_session_start_hook` when `.claude/` exists. (~20 LOC)
- [ ] Coordinate `BLOCK_VERSION` bump across test fixtures in `tests/cli.rs`. (~50 LOC)

## Work Unit 3: Checkpoint & Doctor Health Gates (`src/commands/workflow.rs` & `doctor.rs`)
- [ ] In `workflow::checkpoint_lines`, probe `repo_state` and emit non-blocking drift warnings if `manifest_drift_count > 0`. (~15 LOC)
- [ ] In `doctor::run`, audit adopted projects with `.claude/` for `has_session_start_hook`. (~20 LOC)
- [ ] Add integration tests in `tests/cli.rs` verifying hook injection, doctor finding, and checkpoint warning. (~80 LOC)

## Work Unit 4: Documentation Alignment & Truthfulness
- [ ] Update `docs/user-guide/zero-step-drift-recovery-explained.md` explaining automated hook vs instruction-driven delivery. (~30 LOC)
- [ ] Update `docs/user-guide/harnesses-loops-and-context-masterclass.md` line 111. (~10 LOC)
- [ ] Update `docs/user-guide/workflow-panel-native-vs-agent-skills.md` line 44 removing obsolete placeholder claims. (~15 LOC)
