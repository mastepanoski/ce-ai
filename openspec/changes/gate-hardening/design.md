# Design
- tests/e2e.rs probe: cfg!(windows) => panic with FAILED-TO-RUN guidance;
  docker info failure/absence => panic naming exit/cause; success proceeds
  to build+run as before, ending with "[E2E] GATE PASSED".
- Makefile: `security` target (install-if-missing cargo-audit --locked,
  then cargo audit); `.PHONY` updated; e2e comment documents fail-closed.
- ci.yml security-audit job: tee output to audit-output.txt and append to
  GITHUB_STEP_SUMMARY under a "## Supply Chain Security Audit" heading;
  `set -o pipefail` preserves cargo-audit's non-zero exit through tee.
