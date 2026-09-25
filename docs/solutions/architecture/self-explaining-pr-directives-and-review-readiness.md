---
module: governance
tags: [pull-requests, code-review, block-version, v6, init-prj, openspec, agents-md, review-readiness, evidence]
problem_type: architecture
title: "Self-Explaining PR Directives & Upfront Review Readiness in AGENTS.md Managed Blocks"
applies_when: "When AI agents open pull requests with mechanical diff summaries rather than decision rationale, missing upfront verification evidence, or before automated checks pass."
---

# Self-Explaining PR Directives & Upfront Review Readiness in AGENTS.md Managed Blocks

## Problem
In AI-assisted software delivery, code diff generation is frictionless, but human review is the primary bottleneck. AI agents (Claude Code, Antigravity, OpenCode, Codex, Cursor) frequently opened Pull Requests that:
1. **Paraphrased Diffs Mechanically**: Described what code changed rather than explaining *what alternatives were explored and ruled out* and *which specific project rule, invariant, or architectural boundary decided the chosen approach*.
2. **Omitted Upfront Verification Evidence**: Expected human reviewers to ask for test logs, reproduction traces, or execution outputs after opening the PR.
3. **Requested Reviews Prematurely**: Marked PRs ready for review before local linters, unit tests, and browser/E2E checks had passed cleanly.
4. **Ignored Subsystem Routing**: Omitted domain owners and `CODEOWNERS` when routing review requests.

## Solution

Adopted the dual paradigm:
- **"A change that can explain itself"**:
  > *"Today a pull request tells you what changed. It should be able to tell you what it ruled out, and which rule decided it."*
- **"And the back half is review"**:
  - **Evidence**: Attached upfront to the pull request in collapsible `<details>` blocks, not asked for later.
  - **Browser / Automated Checks**: Run to 100% green before a human opens it.
  - **Routing**: Changes directed to whoever knows that area (`CODEOWNERS`).

### Architectural Components

1. **Managed Adoption Block Templates (`render_block_content` in `src/commands/init_prj.rs`):**
   - Extended `AdoptionTier::Full`, `Minimal`, and `Orchestrator` to include explicit Stage 7 PR & Review Readiness directives.
   - Mandated documenting *What It Ruled Out* and *Which Rule Decided It*, attaching upfront evidence in collapsible blocks, running automated checks first, and routing reviewers.

2. **Managed Block Version Bump (`BLOCK_VERSION: 6`):**
   - Bumped `BLOCK_VERSION` 5 $\to$ 6 in `src/commands/init_prj.rs` and coordinated `CUR_BLOCK_VERSION = 6` in `tests/cli.rs`.
   - Projects adopted under older versions are classified as `StaleVersion` by `ce-ai doctor` and `ce-ai status`, offering an actionable upgrade command (`re-run ce-ai init-prj --tier <tier> to upgrade`).

3. **LOC Budget & PR Size Policy Analysis:**
   - Evaluated the interaction between self-explaining PRs and the 400-line review boundary in `CONTRIBUTING.md`.
   - **Ruling**: The 400 LOC boundary must NOT be relaxed. Small, atomic PRs are a prerequisite for clear self-explanation because they maintain a tight decision surface.
   - **Collapsible Disclosure Widgets**: Raw logs and terminal traces must use markdown `<details><summary>` blocks, keeping the PR description within its ~100–150 line cognitive budget without inflating code line counts (`git diff --numstat`).

4. **Repository Governance Updates:**
   - Updated root `AGENTS.md` Invariant #7 and Definition of Done (DoD).
   - Documented PR sizing and collapsible evidence rules in `CONTRIBUTING.md`.
   - Refreshed root `AGENTS.md` and `GEMINI.md` adoption blocks to v6 (`sha256=0b96d615...`).
