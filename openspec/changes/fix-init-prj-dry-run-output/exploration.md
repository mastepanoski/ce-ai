# Exploration: `init-prj` dry-run semantics

## Question
Why does `ce-ai --dry-run init-prj` print an adoption success message and omit the RTK hook actions that a real run performs?

## Findings
- `src/commands/init_prj.rs` `run()` wraps **all** side effects in `if !ctx.dry_run { ... }`.
- The final reporting block is **outside** that guard and unconditional, so it prints `✓ Adopted project at ...` even when nothing was written.
- `configure_rtk_hook` (`src/harness/rtk.rs:104`) is already dry-run aware and prints `[dry-run] would configure rtk hook for <harness>` when `dry_run=true`.
- `reconcile_rtk_hooks_if_supported` (`src/commands/init_prj.rs:392`) is only called inside `!ctx.dry_run`, so its preview never executes in dry-run mode. This is the "omits planned hook actions" defect.
- Correct precedent already exists: `backups.rs:75-80` early-returns with `dry-run: would restore backup to <path>`.
- Non-dry-run-aware writers (must stay guarded): `write_atomic` (AGENTS.md), CLAUDE.md stub write, `ensure_gitignore_block`, `reconcile_project_harness_hooks`, `SkillRegistry::sync_registry`, `init_codegraph_if_available`, `state.save`.

## Options Evaluated
- **A — Messaging-only:** change the final print. Fixes the "lies about adoption" criterion but leaves the omitted-hook defect the issue title calls out. Superficial.
- **B — Messaging + invoke dry-run-aware RTK reconciliation:** low risk, addresses both defects, leaves non-dry-run-aware writers untouched.
- **C — Full dry-run awareness across all reconcile helpers:** thread `dry_run` into every harness write path. Architecturally ideal but a refactor with a large blast radius, disproportionate to a bug fix.

## Decision
**Option B.** A is a symptom patch forbidden by AGENTS.md; C is scope creep for a defect fix. The remaining gap (preview of project rule/stub/gitignore actions) is documented as a follow-up rather than silently ignored.
