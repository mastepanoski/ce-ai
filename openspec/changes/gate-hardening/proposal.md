# Proposal: `gate-hardening`

## Why
Issue #165 (P1): verification gates must verify.
1. `make e2e` reported PASS with Docker unavailable — the container scenario
   silently skipped (`println!("SKIPPED")` + early return).
2. No local supply-chain gate existed: `cargo audit` only ran in CI.

## What Changes
- E2E gate becomes fail-closed: Docker absence/Windows host is a hard
  failure with remediation guidance instead of a silent skip.
- New `make security` target: installs cargo-audit if missing and fails on
  any advisory — local parity with the CI Supply Chain job.
- CI security job writes its output to the GitHub step summary so "green"
  always means "executed".
