# Tasks: Installer Download Resilience

- [ ] Unit 1: `scripts/install.ps1` — retry loop (3 attempts, 10s/20s backoff) around resolve+download; recent-releases fallback; loud final error.
      - Verification: Windows PowerShell Installer Gate green on this PR (runs the real script).
- [ ] Unit 2: `scripts/install.sh` — mirror retry/fallback semantics with size verification.
      - Verification: shellcheck-clean logic review; manual dry syntax check (`bash -n`).
- [ ] Unit 3: Full CI matrix green on the PR.
