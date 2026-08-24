# Proposal: `tui-upgrade-contract-fix`

## Why
The TUI "Upgrade Release" panel still spawned
`ce-ai upgrade --harness all --force`, flags removed from the CLI contract in
v1.18.1 (Issue #161) — so the action failed with clap exit 2 for every user.

## What Changes
- TUI spawns plain `upgrade` (reconciles all active harnesses by design);
  obsolete "Target Harness Scope" line removed.
- Anti-drift net: every TUI-spawned vector extracted into a pure builder and
  unit-validated against its live subcommand clap surface via augment_args —
  any future CLI contract change fails these tests instead of runtime.
