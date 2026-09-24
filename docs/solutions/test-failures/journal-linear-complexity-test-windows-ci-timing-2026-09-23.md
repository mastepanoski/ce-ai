---
title: Calibrating journal linear complexity test wall-clock bounds for Windows CI runners
date: 2026-09-23
category: test-failures
module: state/journal.rs + state/tests/journal.rs
problem_type: test_failure
component: testing_framework
severity: low
symptoms:
  - "state::journal::tests::journal_arm_complexity_is_strictly_linear fails intermittently on windows-latest runner in CI"
  - "Panics with: 500 arm() calls took 10.05s, expected < 10s for linear complexity"
root_cause: timing_jitter
resolution_type: test_fix
tags:
  - "journal"
  - "windows-ci"
  - "fsync"
  - "timing"
  - "flaky-test"
applies_when: "When tests with synchronous disk durability (fsync/FlushFileBuffers) fail intermittently on virtualized CI runners under heavy I/O contention."
---

# Calibrating journal linear complexity test wall-clock bounds for Windows CI runners

## Problem
In `src/state/tests/journal.rs`, `journal_arm_complexity_is_strictly_linear` verifies that mutation arming executes in linear $O(N)$ time with $O(1)$ write amplification per operation, guarding against the severe quadratic $O(N^2)$ serialization regression (> 10 minutes).

The test executes 500 `j.arm()` calls, each invoking `sync_all()` (`FlushFileBuffers` on Windows). On virtualized GitHub Actions Windows runners (`windows-latest`), shared storage latency occasionally causes individual flushes to take ~20ms, resulting in 500 calls taking 10.05 seconds. A rigid assertion of `< 10` seconds produced a false-positive failure on `main` post-merge.

## Root Cause
Windows NTFS runners in multi-tenant CI environments have variable I/O latency for physical file synchronization. The test already proves linear algorithmic complexity mathematically via:
1. `max_delta < 3_000` (proves $O(1)$ write amplification per operation).
2. `final_size < 1_500_000` (proves $O(N)$ linear journal size).

The wall-clock assertion was designed solely as an empirical guardrail against the 10-minute quadratic regression, but its threshold was set too tight (`< 10s`), leaving zero margin for Windows runner I/O jitter.

## Solution
Relax the wall-clock assertion from `< 10` seconds to `< 30` seconds in `src/state/tests/journal.rs`:
```rust
assert!(
    elapsed.as_secs() < 30,
    "500 arm() calls took {:?}, expected < 30s for linear complexity (quadratic took >10m)",
    elapsed
);
```

30 seconds provides 3x margin for Windows NTFS runner latency while remaining 20x faster than the 10-minute quadratic regression.
