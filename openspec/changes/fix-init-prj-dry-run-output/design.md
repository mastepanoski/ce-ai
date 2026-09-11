# Design: `init-prj` dry-run reporting

## Control flow in `run()` (`src/commands/init_prj.rs`)
1. Resolve `target_dir`, `tier`, existing AGENTS.md content, `full_block`, `body_sha256`, and `(new_content, is_already_up_to_date)` — **unchanged**.
2. Load global `State` **unconditionally** (read-only) before branching, so both modes can consult `installed_harnesses`. `State::load` returns `State::new()` when the file is absent.
3. Real path (`!ctx.dry_run`) — unchanged ordering and messages:
   write AGENTS.md (if not up to date) → derived `CLAUDE.md` stub → register project entry → `ensure_gitignore_block` → `reconcile_project_harness_hooks` → `SkillRegistry::sync_registry` → `init_codegraph_if_available` → `reconcile_rtk_hooks_if_supported` (unless opted out) → `state.save`.
4. Dry-run path (`ctx.dry_run`) — no writes:
   if RTK is not opted out, call `reconcile_rtk_hooks_if_supported`, which delegates to `configure_rtk_hook(home, harness, dry_run=true, quiet)` and prints `[dry-run] would configure rtk hook for <harness>`; otherwise print the opt-out notice.
5. Final reporting:
   - `is_already_up_to_date` → existing "already adopted with up-to-date block" message, return `Ok(())`.
   - `ctx.dry_run` → `dry-run: would adopt project at '<path>' (tier: <tier>, block SHA: <sha8>)`, return `Ok(())`.
   - else → `✓ Adopted project at '<path>' (tier: <tier>, block SHA: <sha8>)`.

## Output contract
| Condition | Final line |
|---|---|
| dry-run, block would change | `dry-run: would adopt project at '<path>' (tier: <tier>, block SHA: <sha8>)` |
| real run, block would change | `✓ Adopted project at '<path>' (tier: <tier>, block SHA: <sha8>)` |
| either, block already current | `Project at '<path>' is already adopted with up-to-date block (SHA: <sha8>).` |

## Invariants
- Dry-run performs zero `write_atomic`, zero `fs::write`, zero external mutating commands, and zero `state.save`.
- Real-run ordering, stdout, and side effects are unchanged.
- `--quiet` suppresses all informational output in both modes.

## Test strategy (TDD)
- Integration (primary, `tests/cli.rs`): run `ce-ai --config-dir <tmp> --dry-run init-prj <tmp/prj> --tier minimal`, assert stdout contains `dry-run: would adopt project` and does **not** contain `Adopted project`; snapshot the project tree and `state.json` before/after and assert byte-identical; assert exit success.
- Unit (`src/commands/tests/init_prj.rs`): keep/extend the existing `reconcile_rtk_hooks_if_supported` dry-run test as the delegation contract.
