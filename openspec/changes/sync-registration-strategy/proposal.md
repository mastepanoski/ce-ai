# Proposal: `sync-registration-strategy`

## Why

PR #206 fixed the sync fallthrough by giving each harness its own arm, but the
arm bodies are ~90% copy-paste: eight vendors repeat the identical
`register_<vendor>_mcp_server(codegraph)` + `(engram)` + managed-skills-copy
sequence with only two degrees of freedom (which registrar function, which
skills subpath). The chain is exactly the shotgun-surgery surface the audit
series kept flagging: adding a harness means editing N command files and any
omission is a silent runtime bug.

## What Changes

- Collapse the re-registration arms in `sync_with()` into a single exhaustive
  **strategy table**: `registration_spec(kind) -> Option<RegistrationSpec>`
  where `RegistrationSpec { register_mcp: Option<McpRegistrar>,
  skills_subpath: Option<&'static str> }`.
- Rust match exhaustiveness makes forgetting a new `HarnessKind` variant a
  compile error instead of a fictional write path.
- No behavioral change: same registrations, same copies, same errors, same
  verification matrix. The 94-test black-box CLI suite is the safety net.

## In Scope

- `src/commands/sync.rs` only (+ unit test for the table).

## Out of Scope

- `install.rs` consolidation (same pattern lives there; separate change once
  this shape proves out in production).
- Trait-object Strategy over adapters: unnecessary here — Rust enum matches
  give compile-time exhaustiveness without dynamic dispatch.

## Success Criteria

- Net deletion of duplicated arm code (~150 lines) with zero behavior delta.
- Full gate suite green; CI matrix green.
