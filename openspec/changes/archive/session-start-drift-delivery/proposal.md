# Proposal: Guaranteed Turn-0 Drift Delivery via Native Lifecycle Hooks and Enforced Directives

## Problem Statement
In `ce-ai v1.30.0`, `workflow resume` implemented live sub-15ms `RepoState` drift detection (Git working tree, branch, manifest SHA256 integrity, and OpenSpec progress). However, invocation was purely manual:
1. No native harness lifecycle hooks (`SessionStart`, etc.) were installed by `ce-ai`.
2. The managed block template in `AGENTS.md` (and mirrored files) lacked Turn-0 session start directives.
3. Runtimes relied on LLM memory to remember running `ce-ai workflow resume`.
4. Documentation overpromised automated delivery by claiming "Autonomous harnesses run `ce-ai workflow resume --json`".

## In-Scope Boundaries
- **Claude Code Native Hook Integration:** `ce-ai init-prj` non-destructively injects a `SessionStart` command hook into `.claude/settings.json` running `ce-ai workflow resume`; `deinit-prj` reverses it cleanly.
- **Universal Textual Directive & Block Version Bump:** Update `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs` to include a mandatory Turn-0 `ce-ai workflow resume` directive. Bump `BLOCK_VERSION` 3 → 4.
- **Runtime Checkpoint & Doctor Gates:**
  - `ce-ai workflow checkpoint` runs `probe_repo_state()` and surfaces drift warnings on stage transitions.
  - `ce-ai doctor` validates that `.claude/settings.json` has the `SessionStart` hook configured if `.claude` exists.
- **Documentation Realignment:** Update `zero-step-drift-recovery-explained.md`, `harnesses-loops-and-context-masterclass.md`, and `workflow-panel-native-vs-agent-skills.md` to truthfully reflect what is automated vs instruction-driven.

## Out-of-Scope Boundaries
- Building arbitrary daemon watchers or background execution wrappers around agent binaries.
- Modifying closed-source IDE internals (e.g. Cursor or VS Code Copilot private APIs).
- Breaking or modifying unmanaged user hooks or settings in `.claude/settings.json`.

## Risk Evaluation & Mitigation
- **Risk: Clobbering user `.claude/settings.json` configurations:**
  *Mitigation:* Parse existing JSON as a `serde_json::Value`, target specifically `hooks.SessionStart`, preserve all other keys, and write atomically using `crate::state::write_atomic`.
- **Risk: Test invalidation due to `BLOCK_VERSION` bump (3 → 4):**
  *Mitigation:* Coordinate bump with `CUR_BLOCK_VERSION` in `tests/cli.rs` and verify all tests pass.

## Success Criteria
1. When `ce-ai init-prj` runs in a directory with `.claude/`, `.claude/settings.json` contains the `SessionStart` hook running `ce-ai workflow resume`.
2. `ce-ai deinit-prj` removes the hook surgically and cleans up the file if it was created by `ce-ai`.
3. `AGENTS.md` contains the Turn-0 directive and declares `v=4`.
4. `ce-ai doctor` reports a finding if the Claude hook is missing on an adopted project.
5. All unit, integration, and E2E gates pass 100% green.
