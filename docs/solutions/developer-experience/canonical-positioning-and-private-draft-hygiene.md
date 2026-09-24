---
title: "Canonical Positioning Alignment and Private Draft Hygiene"
date: "2026-09-24"
category: "developer-experience"
module: "docs"
component: "documentation"
severity: "medium"
problem_type: "documentation_hygiene"
symptoms:
  - "Private community feedback issue draft published into repository documentation map"
  - "Inconsistent definitions of CE-AI describing it as a harness rather than an engineering workflow for coding agents"
  - "ce-ai doctor warning indicating active change lacks target domain mapping in spec.md"
root_cause: "A discussion draft intended for external upstream communication was erroneously committed to public docs and linked in the README, while internal documentation still used legacy 'engineering harness' terminology rather than the canonical definition: 'CE-AI is not another coding agent. It is an engineering workflow for coding agents.' Additionally, the OpenSpec spec.md lacked required YAML frontmatter with title and domain."
resolution_type: "documentation_fix"
tags: [developer-experience, documentation, positioning, openspec, hygiene]
applies_when: "When aligning repository documentation with canonical product positioning, ensuring private community outreach drafts stay outside public documentation, or fixing OpenSpec spec.md frontmatter validation."
---

# Canonical Positioning Alignment and Private Draft Hygiene (Release v1.68.3)

## Problem

1. **Private Upstream Discussion Draft Exposure**: A discussion draft intended for manual communication to the upstream EveryInc/compound-engineering-plugin community was committed as `docs/community/compound-engineering-discussion-draft.md` and linked in `README.md`. Because upstream community outreach is handled directly by the project maintainer, publishing this internal draft in the public documentation map created confusion.
2. **Terminology Drift on CE-AI Definition**: Several documentation surfaces (`README.md`, `ce-ai-positioning.md`, `quick-start-workflow-guide.md`, `harnesses-loops-and-context-masterclass.md`) referred to CE-AI as an "engineering harness" rather than adhering to the canonical positioning: *"CE-AI is not another coding agent. It is an engineering workflow for coding agents."*
3. **OpenSpec Domain Frontmatter Omission**: `openspec/changes/reposition-public-messaging/spec.md` lacked YAML frontmatter with `title` and `domain`, causing `ce-ai doctor` to report that the change lacked target domain mapping.

## Solution

1. **Unpublished Private Community Draft**:
   - Removed `docs/community/compound-engineering-discussion-draft.md` and deleted the `docs/community/` directory.
   - Removed the discussion draft link row from the documentation map in `README.md`, keeping `README.md` at 89 lines (well under the 100-line budget).

2. **Unified Canonical Positioning Across Surfaces**:
   - Updated `README.md` table answers ("What is CE-AI?" and "How is it different?") to reinforce:
     - *"An open-source engineering workflow for coding agents that coordinates work, verification, and durable learning around the agent you already use."*
     - *"Claude Code, Codex, and OpenCode are agents/hosts. Compound Engineering is the methodology and skill plugin. CE-AI is the engineering workflow for coding agents around them."*
   - Refined `docs/user-guide/ce-ai-positioning.md`, `docs/user-guide/quick-start-workflow-guide.md`, and `docs/user-guide/harnesses-loops-and-context-masterclass.md` to consistently refer to CE-AI as the engineering workflow for coding agents.

3. **Frontmatter Compliance**:
   - Added standard frontmatter (`title: Reposition Public Messaging`, `domain: workflow`, `version: 1.0.0`) to `openspec/changes/reposition-public-messaging/spec.md`, clearing the `ce-ai doctor` warning.
