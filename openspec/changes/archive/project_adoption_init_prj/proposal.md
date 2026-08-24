# OpenSpec Proposal: Project Adoption via Managed AGENTS.md Workflow Blocks (`init-prj` / `deinit-prj`)

## Problem Statement
Today `ce-ai install` only mutates global surfaces (`opencode.json`, managed plugin directory `~/.config/opencode/compound-engineering`). Skills such as `ce-brainstorm` and `ce-plan` are opt-in per session: without explicit governance rules in a target repository's `AGENTS.md`, nothing obligates AI agents to follow the mandatory 7-stage Compound Engineering development cycle or OpenSpec workflow. 

To solve this, `ce-ai` requires a dedicated, non-destructive **Project Adoption Engine** (`ce-ai init-prj` and `ce-ai deinit-prj`) that injects and manages marker-delimited workflow blocks in project instruction files (`AGENTS.md` and derived stubs) without clobbering user-owned content.

## In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **CLI Commands (`ce-ai init-prj` & `ce-ai deinit-prj`)**:
  - `ce-ai init-prj [PATH]`: Injects versioned, marker-delimited managed blocks into `AGENTS.md` (and derived reference stubs like `CLAUDE.md`). Supports `--tier full|minimal|orchestrator` and `--force`.
  - `ce-ai deinit-prj [PATH]`: Symmetrically removes injected blocks. If `AGENTS.md` was created by `ce-ai` and is empty after extraction, deletes the file cleanly; otherwise preserves 100% of user content.
- **Harness Adapter Extension (`HarnessAdapter` Trait)**:
  - Extend `HarnessAdapter` to define instruction file contracts (`AGENTS.md` canonical file, derived reference stubs, adoption status probing) across all target harnesses (`OpenCode`, `Claude Code`, `Codex`, `Cursor`, `Copilot`, `Pi`, `Custom`).
- **Project Adoption Registry in `state.json`**:
  - Track adopted projects (`path`, `file`, `tier`, `block_version`, `block_sha256`, `created_file`) atomically via `write_atomic`.
- **TUI Dashboard Integration**:
  - Add interactive `[I] Init Project` and `[D] Deinit Project` action shortcuts in `ce-ai tui`.
- **OpenCode Slash Command & Skill**:
  - Package `/ce-ai-init-prj` slash command and `ce-ai-init-prj` skill for agent-driven adoption.
- **Companion Probes in `status` and `doctor`**:
  - `ce-ai status` and `ce-ai doctor` audit project adoption state, detect manual edits inside managed markers (SHA drift), and report per-harness compliance.

### Out-of-Scope:
- Mutating project repositories automatically during global `ce-ai install` (violates least surprise).
- Overwriting unmanaged user content outside marker boundaries.

## Risk Evaluation & Risk Matrix
| # | Risk | Impact | Mitigation |
|---|------|--------|------------|
| R1 | Clobbering user configuration/content | High | Marker-delimited managed block only (`<!-- ce-ai:block begin v=1 -->`); append/replace-inside-markers policy; golden test `init-prj` ➔ `deinit-prj` restores original bytes. |
| R2 | Manual edits inside the managed block get silently overwritten | Medium–High | Registry SHA mismatch ➔ refuse overwrite and require `--force`; `ce-ai doctor` reports drift. |
| R3 | Content drift between `AGENTS.md` and `CLAUDE.md` copies | Medium | Single canonical file (`AGENTS.md`); `CLAUDE.md` is a one-line reference stub, never duplicated content. |
| R4 | Orphaned/inconsistent blocks after global `uninstall` or plugin upgrade | Medium | Adoption registry in `state.json`; `ce-ai uninstall` warns; `ce-ai sync` refreshes stale `block_version`s. |
| R5 | Over-rigid "MUST" language polluting small-change workflows | Medium | Tier system (`full`, `minimal`, `orchestrator`) with scope-gated wording; opt-in per project. |
| R6 | Crash mid-write leaves truncated instruction file | High | All mutations through `write_atomic` (tempfile + rename); ISO 27002 CP-9 alignment. |

## Success Criteria
1. `ce-ai init-prj` injects tier-specific managed blocks into `AGENTS.md` using `<!-- ce-ai:block begin -->` markers cleanly.
2. `ce-ai deinit-prj` removes managed blocks and restores `AGENTS.md` byte-for-byte to pre-init state.
3. `ce-ai status` and `ce-ai doctor` report project adoption state and SHA drift accurately.
4. TUI dashboard actions (`[I]` / `[D]`) trigger init/deinit workflows smoothly.
5. All unit, CLI integration, and containerized E2E tests pass 100% green across Linux, macOS, and Windows.
