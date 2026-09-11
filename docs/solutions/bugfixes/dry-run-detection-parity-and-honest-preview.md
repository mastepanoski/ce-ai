---
title: "Dry-Run Detection Parity & Honest Preview Output"
category: "bugfixes"
module: "src/commands/"
tags: ["dry-run", "preview", "detection-parity", "bugfix", "init-prj"]
problem_type: "bug"
severity: "P2"
---

# Dry-Run Detection Parity & Honest Preview Output

## Problem
`ce-ai init-prj --dry-run` was not a faithful preview of a real run:
1. **False success claim**: it printed the real `✓ Adopted project ...` line although nothing was written, because the final reporting block sat outside the `if !ctx.dry_run` guard and had no dry-run branch.
2. **Omitted actions**: it skipped the RTK hook reconciliation entirely, so no `[dry-run] would configure rtk hook for <harness>` lines appeared, understating impact.
3. **Detection drift**: even after invoking reconciliation, a fresh project previewed *no* Claude hook, because the real run's detection depends on the `CLAUDE.md` stub that a real run writes first but a dry-run does not.

Discovered by dogfooding `init-prj` on the `ce-ai` repo itself (issue #351).

## Root Cause
- The success print was unconditional: dry-run fell through to `✓ Adopted project`.
- `reconcile_rtk_hooks_if_supported` was only called inside `!ctx.dry_run`, although `configure_rtk_hook` (`src/harness/rtk.rs`) already short-circuits on `dry_run` before any write.
- **Detection parity**: the Claude branch of `reconcile_rtk_hooks_if_supported` triggers on `.claude exists || CLAUDE.md exists || has_installed("claude")`. A real `init-prj` always creates the derived `CLAUDE.md` stub before reconciliation, so Claude always matched; dry-run wrote nothing, so the guard evaluated differently than the run it was previewing. A preview that changes its own inputs is not a preview.

## Solution
1. **Honest preview line** — replace the unconditional success print with three explicit modes:
```rust
if is_already_up_to_date { /* "already adopted" */ return Ok(()); }

if ctx.dry_run {
    println!("dry-run: would adopt project at '{}' (tier: {}, block SHA: {})", /* ... */);
    return Ok(());
}

println!("✓ Adopted project at '{}' (tier: {}, block SHA: {})", /* ... */);
```

2. **Preview the same actions a real run performs** — invoke the already dry-run-aware reconciliation in dry-run mode:
```rust
} else if !crate::harness::rtk::is_rtk_opted_out(skip_rtk, skip_companions) {
    // dry-run preview: writes nothing, prints [dry-run] would configure ...
    reconcile_rtk_hooks_if_supported(&target_dir, &state, ctx, true)?;
}
```

3. **Parity flag for the stub guarantee** — pass `claude_rule_expected` so the preview mirrors the real run's post-write state:
```rust
fn reconcile_rtk_hooks_if_supported(target_dir, state, ctx, claude_rule_expected: bool)
```
Real path passes `false` (the stub physically exists by then); dry-run passes `true` (the stub *would* be created).

4. **Keep the write-free path write-free and robust** — the real path keeps `State::load` after the writes (unchanged ordering, fail-fast on corrupt state); the dry-run path performs a best-effort read so unrelated global-state corruption cannot abort a preview that writes nothing.

## Key Pattern
Preview modes must compute their output from the **post-run** state, not the pre-run state. If a real run creates a file that gates later detection, the preview must model that file as present. Keep dry-run strictly non-mutating, and never let a preview be less informative than the run it previews.

## Prevention
- Any `--dry-run` path must have its own explicit branch and a distinct message; never share the success print with the real path.
- Add a regression test that runs `--dry-run` against a **fresh** project (no pre-created harness dirs) and asserts the same harness set the real run would configure. Pre-creating `.claude` in the test hides detection-parity bugs.
- Test the invariant, not just the output: snapshot the target and `$HOME` before/after and assert zero writes (including external hook config).
- Cover `--quiet` and the already-up-to-date + dry-run combination, which have independent output branches.
