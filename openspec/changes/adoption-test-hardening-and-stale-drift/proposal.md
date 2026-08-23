# Proposal: Adoption v2 Test Hardening & Stale-Block Doctor Signal

## Problem Statement

Post-release review of adoption block v2 surfaced coverage gaps and an operator-experience gap:

1. Only CRLF fixtures exercise the v1→v2 replacement path; LF-only repos are untested.
2. Nothing verifies the header `sha256=` matches the actual body hash (a systematically wrong derivation would ship green while silently breaking doctor/status drift detection).
3. No test proves the malformed-block error path (begin marker without end marker → `CeError::Runtime`).
4. When adopted projects run stale v1 blocks, `doctor` reports a generic "block SHA drift" — indistinguishable from tampering — instead of telling operators exactly what to do.

## In Scope

- New integration tests: LF-only replacement variant; malformed-block error path; header-sha256 ↔ body ↔ state triangle consistency.
- `doctor`: parse managed-block version from the on-disk header; when it is older than `BLOCK_VERSION`, emit a targeted "stale block version … re-run ce-ai init-prj --tier <tier>" finding instead of generic SHA drift.

## Out of Scope

- `status.rs` wording parity (same underlying data; follow-up if operators ask).
- Auto-upgrade behavior in doctor (doctor diagnoses; init-prj fixes).

## Risk

Low: doctor change is message-only for the mismatch branch; new tests are additive.

## Success Criteria

- All four scenarios covered by green integration tests; full suite + gates pass.
