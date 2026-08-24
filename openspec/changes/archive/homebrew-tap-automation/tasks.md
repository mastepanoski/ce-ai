> STATUS (v1.20.1): Formula/ce-ai.rb and tag-triggered release pipeline live. Residual open boxes below were not re-audited item-by-item.

# Tasks: Automated Homebrew Tap Updates

## 1. Tap self-update workflow (lives in homebrew-ce-ai)

- [x] 1.1 Create `.github/workflows/self-update.yml` in the tap repo with
      30-minute cron and `workflow_dispatch` triggers
- [x] 1.2 Implement latest-release polling + version comparison skip
- [x] 1.3 Implement asset download + SHA256 computation for the four
      platform tarballs
- [x] 1.4 Implement formula rendering from heredoc template, commit, and
      push using the tap's own `GITHUB_TOKEN` (no PAT)

## 2. Verification

- [x] 2.1 YAML syntax validated; heredoc render compared byte-for-byte
      against the working v1.0.8 tap formula (identical)
- [x] 2.2 Manual dispatch run completed successfully on
      homebrew-ce-ai with zero secrets configured (idempotent skip path)
- [ ] 2.3 End-to-end: publish next real release and confirm the tap
      receives the automated bump and `brew upgrade` picks it up

## 3. Documentation

- [x] 3.1 No secrets required: automation uses the tap's own
      `GITHUB_TOKEN`; document in PR description
