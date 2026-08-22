# Functional Specification: Context-Exhaustion Resilience

## Requirement 1: GitHub Branch Protection Health Probe
**WHEN** `ce-ai doctor` is executed within a git repository tracked on GitHub,  
**THEN** it SHALL query GitHub branch protection rules for `main` via `gh api repos/{owner}/{repo}/branches/main/protection`.  
**WHEN** protection is disabled or missing required status checks,  
**THEN** `ce-ai doctor` SHALL report a `branch-protection: missing or unconfigured` finding and exit non-zero.

## Requirement 2: Local Git Hooks Configuration Health Probe
**WHEN** `ce-ai doctor` is executed,  
**THEN** it SHALL check whether `core.hooksPath` is set to `.githooks`.  
**WHEN** `core.hooksPath` is unconfigured,  
**THEN** `ce-ai doctor` SHALL report a `git-hooks: core.hooksPath not set to .githooks` finding.

## Requirement 3: Automated Branch Protection Configuration Script
**WHEN** `scripts/protect-branch.sh` is executed,  
**THEN** it SHALL invoke the GitHub REST API (`gh api PUT repos/{owner}/{repo}/branches/main/protection`) to enable required PRs, enforce status checks, block force pushes, and disallow deletion of `main`.

## Requirement 4: Compact AGENTS.md Invariant Index
**WHEN** `AGENTS.md` is inspected,  
**THEN** it SHALL feature a compact Hard-Gate Invariant Index (<= 30 lines) at the top of the file specifying non-negotiable imperative constraints before detailed architecture sections.
