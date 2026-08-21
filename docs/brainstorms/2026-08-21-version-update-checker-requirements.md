# Version Update Checker & Recommendation Requirements (Issue #14)

## 1. Problem Statement
When harnesses are installed from a local dev tree (`source: local`), `ce-ai status` and the TUI report `version: local`. Users cannot easily tell if their local installation is up-to-date or lagging behind the latest GitHub release of `compound-engineering-plugin`.

## 2. Key Requirements
1. **Automated Upstream Release Checker (`VC-1`)**:
   - `ce-ai status` and the TUI `Status & Harnesses` tab MUST check the latest GitHub release tag for `compound-engineering-plugin` (caching checks in `state.json` to prevent API rate limits).
2. **Local Source Version Transparency (`VC-2`)**:
   - When a harness has `source: local`, `ce-ai status` and the TUI MUST explicitly indicate that the harness is running from a local source tree AND display the latest available GitHub release tag alongside it.
3. **Upgrade Recommendation Prompt (`VC-3`)**:
   - If a harness is on `local` source or lagging behind the latest release tag, `ce-ai status` and TUI MUST display an upgrade recommendation:
     `💡 Recommendation: Run 'ce-ai upgrade' to update to latest release vX.Y.Z.`

## 3. Success Criteria
- Running `ce-ai status` clearly distinguishes `source: local` from upstream release versions and displays the latest available GitHub release.
- TUI `Status & Harnesses` displays active update recommendations when a newer release is available.
