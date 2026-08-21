# OpenSpec Proposal: Version Update Checker & Recommendations

## Problem Statement
Closes #14. Harnesses with `source: local` display `version: local`, making it difficult for users to know whether they are running the latest upstream release or lagging behind.

## Proposed Changes
1. **Automated Upstream Release Checking**: Probe GitHub releases API (with cached timestamps in `state.json`) in `ce-ai status` and TUI.
2. **Upstream Release Display**: Show the latest release version alongside local/installed versions.
3. **Upgrade Recommendation**: Prompt users to run `ce-ai upgrade` when local or older versions are detected.
