---
module: src/commands/workflow.rs, src/commands/gate.rs
date: '2026-10-19'
problem_type: architecture
category: architecture
component: workflow_engine
severity: medium
symptoms:
  - "ODD tasks subsystem (odd/tasks/) was never used in real development, causing phantom file confusion"
  - "ce-ai workflow graduate violated Invariant 6 (No Semantic Authority) by inventing an artificial workflow gateway"
  - "Workflow status and guide banners referred to ODD rather than native Compound Engineering primitives"
root_cause: architectural_redundancy
resolution_type: architectural_simplification
tags:
  - workflow
  - odd
  - compound-engineering
  - fast-path
  - no-semantic-authority
title: "Decommission ODD Tasks Subsystem and Align Compound Agile Fast-Path"
applies_when: "When executing lightweight bug fixes, tactical tasks (<200 LOC), or resolving workflow track confusion."
---

# Decommission ODD Tasks Subsystem and Align Compound Agile Fast-Path

## Problem

`ce-ai` originally experimented with Organic Driven Development (ODD)—an agile concept created by Alan Buscaglia in Gentle AI—by implementing a custom, standalone `odd/tasks/<feature>.md` file format and a `ce-ai workflow graduate` command to promote briefs into formal OpenSpec change directories (`openspec/changes/`).

In practice:
1. **Zero Real-World Adoption**: In real development across multiple production releases, `odd/tasks/` was never created or used. Real-world lightweight planning already happens via `docs/plans/` (`/ce-plan`) or direct execution (`/ce-work`, `/ce-debug`).
2. **Semantic Authority Violation**: Inventing a proprietary `odd/tasks/` format and requiring a `graduate` bridge violated Invariant 6 of `AGENTS.md` (*No Semantic Authority: CE-AI must not require artifacts that Compound Engineering itself does not require*).
3. **Cognitive Overhead**: Developers and AI agents were confused by CLI guidance prompting `odd/tasks/` creation and graduation rather than leveraging standard Compound Engineering primitives.

## Solution

To eliminate redundant ceremony and align `ce-ai` strictly with Compound Engineering contracts:

1. **Compound Agile Fast-Path**:
   - Re-aligned the fast-path in `ce-ai workflow status` and `guide` to the **Compound Agile Fast-Path** (`docs/plans/`, `/ce-work`, `/ce-debug`).
   - For bug fixes, chores, and tactical tasks (<200 LOC), agents execute directly or author lightweight plans under `docs/plans/` with zero ceremonial friction.
2. **Deprecation of `graduate`**:
   - Deprecated `ce-ai workflow graduate <feature>` with migration notices while preserving its execution logic for backward compatibility with legacy briefs.
3. **Advisory Gate Check Guidance**:
   - Updated `ce-ai gate check` advisory warnings for diffs exceeding the 200 LOC ceiling to recommend formalizing via `openspec/changes/<feature>/` or structuring via `docs/plans/`, without mentioning `graduate`.
4. **Backward Compatibility**:
   - Preserved `ExecutionMode::Organic` in `state.json` schema deserialization to avoid breaking existing configurations and test fixtures.
