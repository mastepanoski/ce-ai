# Design: `init-prj` dry-run reporting

## Control flow in `run()` (`src/commands/init_prj.rs`)
1. Resolve `target_dir`, `tier`, existing AGENTS.md content, `full_block`, `body_sha256`, and `(new_content, is_already_up_to_date)` — **unchanged**.
2. Real path (`!ctx.dry_run`) — ordering preserved exactly:
   write AGENTS.md (if not up to date) → derived `CLAUDE.md` stub → load global `State` → register project entry → `ensure_gitignore_block` → `reconcile_project_harness_hooks` → `SkillRegistry::sync_registry` → `init_codegraph_if_available` → `reconcile_rtk_hooks_if_supported(..., claude_rule_expected=false)` (unless opted out) → `state.save`.
   `State::load` stays after the writes, so a corrupt `state.json` still fails fast on a real run with no partial write.
3. Dry-run path (`ctx.dry_run`) — no writes. Best-effort registry read (`State::load(...).unwrap_or_default()` with a non-quiet warning) so unrelated global-state corruption cannot abort a write-free preview. If RTK is not opted out, call `reconcile_rtk_hooks_if_supported(..., claude_rule_expected=true)`, which delegates to `configure_rtk_hook(home, harness, dry_run=true, quiet)` and prints `[dry-run] would configure rtk hook for <harness>`; otherwise print the opt-out notice.
4. Final reporting:
   - `is_already_up_to_date` → existing "already adopted with up-to-date block" message, return `Ok(())`.
   - `ctx.dry_run` → `dry-run: would adopt project at '<path>' (tier: <tier>, block SHA: <sha8>)`, return `Ok(())`.
   - else → `✓ Adopted project at '<path>' (tier: <tier>, block SHA: <sha8>)`.

## Detection parity (`claude_rule_expected`)
A real run always leaves the derived `CLAUDE.md` stub in place, so `reconcile_rtk_hooks_if_supported` already detects Claude on every adopted project (via the `CLAUDE.md exists` branch) — but only *after* the stub is written. In dry-run nothing is written, so the raw detection misses Claude on a fresh project. The new `claude_rule_expected: bool` parameter carries that guarantee: `false` on the real path (the stub physically exists by then), `true` on the dry-run path (the stub *would* be created). This is parity, not a new real-run behavior.

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
