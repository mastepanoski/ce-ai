# Proposal: Align CLI audit findings with delivered behavior

## Problem

The CLI audit found no unimplemented top-level dispatches, but several command
claims, accepted flags, and user-facing examples do not match runtime behavior.
`usage report` accepts filters it ignores; `gate check` is described as
observe-only although its default policy can block; `guard` is state-only; and
companion-tool registration is described as installation. The user guides also
contain invalid command examples and omit a discoverable command reference.

## In scope

- Make `usage report` apply documented date filters and reject unsupported
  grouping values instead of silently ignoring them.
- Make help, comments, and user documentation truthful about Claude-only usage
  capture, gate enforcement, guard configuration, companion registration, and
  audit scope.
- Remove the silent `guard disable --harness` mismatch by validating the stored
  scope.
- Add a concise CLI reference and repair invalid user-guide examples.

## Out of scope

- New usage adapters for OpenCode, Codex, Pi, or other harnesses.
- Installing third-party binaries or changing Engram/CodeGraph ownership.
- Designing a new enforcement mechanism for pedagogical guardrails.
- Changing gate policy or its default exit-code behavior.

## Success criteria

1. Date filters change `usage report` output, and unsupported `--by` values
   fail with a usage error rather than silently doing nothing.
2. CLI help never calls a blocking default gate observe-only, a state marker an
   enforced guardrail, or MCP registration binary installation.
3. User documentation has no known invalid examples from the audit and links
   an accurate command reference.
