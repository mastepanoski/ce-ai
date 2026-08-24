# Spec Delta
- **WHEN** Docker is unavailable or unreachable, **THEN** `make e2e` MUST
  fail loudly (non-zero) with FAILED-TO-RUN remediation guidance — never
  report PASS by skipping.
- **WHEN** `make security` runs without cargo-audit installed, **THEN** it
  MUST install it (--locked) before auditing.
- **WHEN** cargo-audit finds vulnerabilities, **THEN** `make security` and
  the CI security job MUST exit non-zero.
- **WHEN** the CI security job runs, **THEN** its full output MUST appear in
  the GitHub step summary.
