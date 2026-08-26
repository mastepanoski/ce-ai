# Exploration: Sync Verification Matrix UX Clarity

## Investigation Summary (2026-08-24)

Root cause of the confusing output, verified against `compound-engineering-v3.23.3`
on a real installation:

1. `ce-ai install` harvests release files only under `.opencode/plugins` and
   `.opencode/skills` (`MANAGED_PREFIXES`, `src/commands/install.rs`).
2. The upstream release ships 33 skills at top-level `skills/` and
   `.agents/skills/`, but **no `.opencode/skills/`** — its own
   `.opencode/INSTALL.md` states the OpenCode plugin registers skills directly
   from the package ("no generated skill copy is required").
3. Therefore `install-manifest.json` contains exactly one file
   (`plugins/compound-engineering.js`) → the opencode surface verifies 1/1.
4. In `ce-ai sync`, `skills_expected` (manifest entries filtered by the
   `skills/` prefix) is empty, so every native harness falls into the
   `NotVerified { reason: "no managed skills tree present" }` arm; cursor is
   registration-only by design (`registration_spec` returns
   `skills_subpath: None`).
5. Sync is reporting faithfully — the empty managed tree is real. The defect
   is purely presentational: the wording implies something was skipped or is
   missing.

## Field Evidence (user machine, 2026-08-24)

- `claude`: CE installed via the Claude Code plugin marketplace
  (`~/.claude/plugins/cache/compound-engineering-plugin`) — outside ce-ai's
  scope, yet the matrix line suggests a ce-ai gap.
- `pi`: 35 `ce-*` skills in `~/.pi/agent/skills` installed by another channel.
- `opencode`: 35 `ce-*` skills in `~/.config/opencode/skills` (user dir), plus
  the ce-ai-managed loader.
- Users reasonably read "unverified" as "your CE install is incomplete".

## Options Considered

### Option A — Reword in place (chosen)

Change the literal strings at the two `println!` sites and the static reasons.

- Pros: smallest diff; no behavior change.
- Cons: rendering stays untestable (`println!` inside `run()`), so the wording
  cannot be pinned by tests; future edits can regress silently.

### Option B — Reword + extract pure render functions (chosen, superset of A)

Extract `matrix_line`, `reconciliation_line`, and the guidance note into pure
functions returning `String`/`Vec<String>`; `run()` prints their results.

- Pros: unit tests pin exact wording (the repo has zero tests over this output
  today); the TUI modal and CLI stay in lockstep by construction; future
  wording changes fail tests instead of drifting.
- Cons: ~30 extra lines of refactoring.

### Option C — Full management model (rejected for this change)

Harvest top-level `skills/` so every harness receives managed skill files and
the matrix verifies them. Correct long-term direction, but it changes install/
sync semantics, manifest shape, and uninstall behavior — a design-level change
requiring its own OpenSpec cycle. Recorded as follow-up, not bundled here
(no drive-by refactors in a bug-fix change).

## Constraints

- `CheckStatus::NotVerified.reason` is `&'static str`; new reasons must be
  static strings (no allocation) — satisfied by literals.
- Exit-code contract untouched: only `Failed` surfaces produce
  `CeError::Verification` (exit 6); `NotVerified` never fails sync.
- The TUI Sync modal renders `run_sync_cmd` output verbatim; multiline note is
  safe there.
