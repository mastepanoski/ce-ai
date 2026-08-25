# OpenSpec Proposal: Sync Verification Matrix UX Clarity

## Problem

`ce-ai sync` prints a Sync Verification Matrix whose wording misleads users —
especially new ones. With the current `compound-engineering` GitHub release
(v3.23.3), which ships plugin-only assets for OpenCode (no managed skills
tree), the matrix reads:

```text
  ✓ opencode: verified — 1/1 files match SHA256
  ○ claude: synced — verification not performed (no managed skills tree present)
  ...
reconciliation status: 1 verified, 9 unverified, 0 failed
```

Three concrete failures:

1. **"synced — verification not performed" is self-contradictory.** A reader
   cannot tell whether the harness is fine or something was skipped.
2. **"9 unverified" reads like an error.** It is actually the expected state:
   ce-ai manages no files on those surfaces, so there is nothing to
   hash-verify. The label conflates "not managed by ce-ai" with "broken".
3. **No guidance.** The output never tells the user what ce-ai manages on each
   surface, how to put a harness under ce-ai management
   (`ce-ai install --harness <name>`), or that CE installed through other
   channels (e.g. the Claude Code plugin marketplace) is outside ce-ai's
   verification scope.

## In Scope

- Reword the per-surface matrix lines: `verified` / `FAILED` / `registered —
  <reason>` with reasons that state *why* nothing was hash-verified.
- Reword the reconciliation summary: `N registered (nothing to verify)`
  instead of `N unverified`.
- Append a short guidance note when any surface is unverified: what
  "registered" means, the command to adopt a harness for management, and the
  scope boundary (other installation channels are not verified).
- Extract matrix rendering into pure, unit-testable functions and pin the new
  wording with tests.
- Update `docs/user-guide/sync-and-upgrade-mechanisms.md` (Step 6) with the
  new output, the three verification states, and a per-harness explanation of
  how to make ce-ai manage each one.
- SemVer patch bump + CHANGELOG entry.

## Out of Scope

- Harvesting the release's top-level `skills/` tree so native harnesses receive
  managed skill files (design-level change; separate OpenSpec change).
- Any change to verification logic, drift detection, or exit codes.
- TUI layout changes (the Sync modal renders the same lines and inherits the
  new wording automatically).

## Risks

- **Downstream parsers**: anything grepping the old strings
  (`verification not performed`, `unverified`) would break. A repo-wide grep
  shows the strings exist only in `src/commands/sync.rs`; docs examples are
  updated in this change. E2E tests do not assert these strings.
- **Noise**: the guidance note adds lines to every non-fully-managed sync. It
  prints only when at least one surface is unverified, and it is the exact
  moment the explanation is needed.

## Success Criteria

- A first-time reader of `ce-ai sync` output can tell, without external docs,
  that `registered` surfaces are healthy and why nothing was hash-verified.
- The output states the command that puts a harness under ce-ai management.
- Unit tests pin every new matrix line and the reconciliation summary.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D
  warnings`, and `cargo test` pass.
