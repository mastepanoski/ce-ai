---
title: "CLI Audit Claims Alignment"
category: "developer-experience"
date: "2026-09-28"
tags:
  - cli
  - documentation
  - usage-ledger
  - guardrail
  - gate
components:
  - commands::usage
  - commands::guard
  - commands::gate
  - commands::tools
  - commands::audit
applies_when: "Auditing a CLI whose help text, accepted flags, and documentation may exceed delivered behavior"
problem_type: "best-practice"
---

# CLI Audit Claims Alignment

## Problem

A command can be wired and parse successfully while still mislead users: flags
can be ignored, a default can contradict its description, and documentation
can retain invalid examples after the interface changes.

## Solution

Treat the executable command surface as the contract. Add focused regression
tests for behavior that previously accepted no-op input, make help text name
the delivered boundary, and maintain a user-facing CLI reference for commands
not covered by task-specific guides.

For this correction, `usage report` applies inclusive RFC 3339 filters and
rejects unsupported grouping; guard disable rejects a mismatching stored
harness scope; and the gate, tools, guard, and audit wording distinguishes
enforcement, configuration, registration, and heuristics respectively.

## Verification

Run targeted command tests, inspect subcommand help, run `ce-ai doc lint
--strict`, and execute the complete Rust and containerized E2E gates.
