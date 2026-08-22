# Tasks: Automated Homebrew Tap Updates

## 1. Workflow

- [x] 1.1 Create `.github/workflows/bump-homebrew.yml` with `release:
      published` and `workflow_dispatch` (input `tag`) triggers
- [x] 1.2 Implement asset download + SHA256 computation for the four
      platform tarballs
- [x] 1.3 Implement formula rendering from heredoc template
- [x] 1.4 Implement tap checkout (`TAP_TOKEN`), commit, and push with
      idempotent empty-commit handling

## 2. Verification

- [x] 2.1 YAML syntax validated; heredoc render compared byte-for-byte
      against the working v1.0.8 tap formula (identical)
- [ ] 2.2 End-to-end: publish next real release and confirm the tap
      receives the automated bump and `brew upgrade` picks it up

## 3. Documentation

- [ ] 3.1 Document `TAP_TOKEN` secret setup (fine-grained PAT,
      Contents read/write on `homebrew-ce-ai`) in the PR description and
      README installation section if needed
