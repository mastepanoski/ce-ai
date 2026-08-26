---
title: Coordinating pinned tests across an adoption-block version bump
date: 2026-08-25
category: test-failures
module: commands/init_prj.rs + tests/cli.rs
problem_type: test_failure
component: testing_framework
severity: medium
symptoms:
  - "Drift-classification test asserts the wrong branch after BLOCK_VERSION bump (StaleVersion instead of DriftDetected)"
  - "Doctor/status tests see no diagnostics for hand-written stale-block fixtures"
  - "Test names referencing concrete versions become factually wrong after each bump"
root_cause: logic_error
resolution_type: test_fix
tags:
  - "adoption-block"
  - "version-bump"
  - "test-fixtures"
  - "doctor-status"
---

# Coordinating pinned tests across an adoption-block version bump

## Problem

Bumping `BLOCK_VERSION` 2→3 (PR #236, commit `5f8bafd`) silently invalidated three classes
of pinned tests in `tests/cli.rs`: classification fixtures, unregistered-fixture
diagnostics, and version-named tests. All three failed or lied at once because they
hardcoded the old version.

## Symptoms

- The tampered-header drift fixture still declared `v=2`, so post-bump it classified as
  `StaleVersion` — never exercising the generic-drift branch it was written for.
- A hand-written stale block in a bare temp dir produced zero `doctor`/`status` output —
  both commands only classify projects registered in `state.json`
  (`src/commands/doctor.rs` iterates `state.projects`).
- `init_prj_replaces_v1_block_with_v2_preserving_content_and_crlf` described a migration
  that no longer exists (the replacement target became v3).

## What Didn't Work

- Editing fixture literals ad hoc per bump — every future bump re-breaks them unless the
  coupling is documented at the edit site.
- Writing only the stale block into a project `AGENTS.md` without adopting: correct bytes,
  but no `state.json` entry means `check_adoption_block_status` is never called for that
  project, so doctor reports nothing and an exit-code assertion on its findings fails.

## Solution

Three coordinated changes in `tests/cli.rs`:

1. **Retarget + pin the coupling inline** (`doctor_reports_generic_drift_for_tampered_current_body`):

```rust
// The literal must match BLOCK_VERSION: an older declared version would
// classify as StaleVersion instead of exercising the generic-drift branch.
tampered.replace_range(begin..line_end,
    "<!-- ce-ai:block begin v=3 tier=full sha256=fedcba -->");
```

2. **Adopt-for-real before simulating staleness**
   (`init_prj_upgrades_stale_v2_block_to_v3_preserving_provenance`): run `init-prj` once so
   the project registers, then overwrite the file with the stale `v=2` block; assert
   `doctor` fails with `stale block version v=2`, then prove re-adoption preserves
   provenance (`created_file == true`) and CRLF endings.

3. **Drop versions from names** (commit `d3bf712`):
   `init_prj_replaces_v1_block_with_v2_...` →
   `init_prj_replaces_stale_v1_block_with_current_preserving_content_and_crlf`.

## Why This Works

`check_adoption_block_status` (`src/commands/init_prj.rs`) classifies by strict precedence:
missing/malformed markers → SHA-match short-circuit to `Ok` → declared `v=` below
`BLOCK_VERSION` yields `StaleVersion` → else `DriftDetected`. A fixture declaring an
outdated version lands one branch lower than intended; only a current-version declaration
plus corrupted SHA reaches generic drift. And because diagnostics iterate `state.json`
entries, a real adoption is what connects any on-disk fixture to the classifier at all.

## Prevention

- **Inline comment discipline**: wherever a test literal encodes `BLOCK_VERSION`, add a
  comment naming the constant and the misclassification risk — the next bumper reads the
  failure site, not the design doc.
- **Adopt-for-real fixture pattern**: every doctor/status test starts with a real
  `init-prj` run, then mutates files; never fabricate blocks without a registered entry.
- **Version-free test names**: describe behavior (`replaces_stale_v1_block_with_current`),
  never source/target versions that rot on every bump.
- **Bump checklist**: when changing `BLOCK_VERSION`, grep `tests/cli.rs` for `v=<n>`
  literals and version numerals in `fn` names within the same commit.

## Known Coverage Gaps (accepted P3/FYI, PR #236 review)

Surfaced by Tier-2 review and deliberately deferred; recorded here so the next bump's
planner inherits them:

- The upgrade lifecycle test exercises `--tier full` only — the orchestrator tier received
  new retention wording in the same diff but has no stale-detection/upgrade-rerun variant,
  and the doctor hint interpolation is only asserted for `full`.
- No test pins the post-bump classification of a v2-era **minimal** adoption (classifies
  `Ok` via the unchanged-body SHA short-circuit, not `StaleVersion`): the R5 byte-parity
  test covers rendering but not the diagnostic consequence.
- Post-upgrade idempotency is unverified: the upgrade rerun asserts success, but no third
  run confirms the already-adopted early-return fires against the v3 block.
- Root `AGENTS.md` and the `full`-tier block share retention meaning but no test pins their
  clauses byte-for-byte against each other — substring assertions allow silent surface
  drift to recur.

## Related Notes

- **Known limitation (discovered, unfixed)**: the SHA short-circuit precedes the version
  check, so a block whose body is byte-identical to the current template classifies `Ok`
  forever regardless of its declared `v=`. PR #236 left the minimal-tier body unchanged, so
  existing minimal-tier v2 adoptions emit no stale-version hint — release-note hint claims
  must be scoped per tier.
- Cross-references: [adoption-block-staleness-alignment-across-status-and-doctor.md](../architecture/adoption-block-staleness-alignment-across-status-and-doctor.md),
  [project-adoption-engine-init-and-deinit-prj.md](../architecture/project-adoption-engine-init-and-deinit-prj.md),
  [init-prj-created-file-clobber-on-re-adoption-2026-08-22.md](../logic-errors/init-prj-created-file-clobber-on-re-adoption-2026-08-22.md)
