---
title: "Managed Adoption Block Hook-Aware Turn-0 Directives & Progressive OpenSpec Alignment"
category: "architecture"
date: "2026-09-26"
tags:
  - adoption-block
  - turn-0
  - session-start-hooks
  - openspec
  - progressive-authoring
  - version-bump
applies_when: "When updating managed AGENTS.md/CLAUDE.md blocks injected into adopted projects or coordinating BLOCK_VERSION bumps."
problem_type: "stale_directive_correction"
---

# Managed Adoption Block Hook-Aware Turn-0 Directives & Progressive OpenSpec Alignment

## Context & Problem
`ce-ai init-prj --tier full` injects a managed governance block (`render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs`) into adopted projects (`AGENTS.md` / `CLAUDE.md`). This text guides AI coding agents working in those repositories.

Two sections of this block became stale and misleading:
1. **Redundant Turn-0 Execution**:
   - The block unconditionally commanded agents to execute `ce-ai workflow resume` at every session start or context compaction.
   - However, `reconcile_project_harness_hooks` (`src/commands/init_prj.rs:486`) automatically installs native `SessionStart` hooks across Claude, Cursor, Codex, Copilot, Pi, and Antigravity that run `ce-ai workflow resume --json` automatically before turn 1.
   - Agents reading the unconditional instruction re-ran the command redundantly, wasting context and tokens.
2. **Premature `tasks.md` Requirement**:
   - The block flatly demanded that all 5 OpenSpec artifacts (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) exist before writing feature code.
   - This contradicted standard progressive authoring: Stage 2 freezes the contract (proposal, exploration, design, spec), while `tasks.md` is derived during Stage 3 (`ce-plan`) from that contract.

## Solution

### 1. Hook-Aware Conditional Turn-0 Directives
Updated the Turn-0 section in `render_block_content(AdoptionTier::Full)`:
- Informs agents that `ce-ai` auto-installs `SessionStart` hooks injecting live FSM state into context before turn 1.
- Directs agents: if state is already present, treat it as current and **do NOT re-run** the command.
- Only run `ce-ai workflow resume` manually when context was not injected (unsupported harness or disabled hook) or when mid-session drift is suspected.

### 2. Progressive OpenSpec Definition
Updated the Stage 2 section in `render_block_content(AdoptionTier::Full)`:
- Clarifies that OpenSpec is authored progressively. Before writing feature code, agents verify the frozen Stage 2 contract (`proposal.md`, `exploration.md`, `design.md`, `spec.md`).
- Clarifies that `tasks.md` is generated in Stage 3 from that frozen contract, and must not be required before Stage 2 is complete, but must be present before opening a PR.

### 3. Coordinated Version Bump (`BLOCK_VERSION: 7`)
- Bumped `pub const BLOCK_VERSION: u32 = 7;` in `src/commands/init_prj.rs`.
- Bumped `const CUR_BLOCK_VERSION: u32 = 7;` in `tests/cli.rs`.
- Added unit test `test_render_block_content_turn_0_and_progressive_openspec` in `src/commands/tests/init_prj.rs`.
- Added integration test `init_prj_upgrades_stale_v6_block_to_v7_with_turn0_and_progressive_openspec` in `tests/cli.rs`.
- Existing `AdoptionTier::Minimal` and `AdoptionTier::Orchestrator` blocks remain unchanged.
