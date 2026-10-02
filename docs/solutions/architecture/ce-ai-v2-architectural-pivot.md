---
module: architecture::v2
tags: [architecture, v2, companion, fleet, upstream-alignment, fsm, native-installers]
problem_type: architecture
title: "CE-AI v2 Architectural Pivot: Cross-Host Operational Companion"
applies_when: "When aligning ce-ai architecture with upstream Compound Engineering boundaries and contracts."
date: 2026-10-01
component: architecture
severity: standard
---

# CE-AI v2 Architectural Pivot: Cross-Host Operational Companion

## Context & Objectives

Following an extensive architectural review from the upstream maintainer of [Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin), `ce-ai`'s architectural boundaries were systematically reassessed. The review identified that `ce-ai` had inadvertently drifted into duplicating upstream responsibilities (file copying, global state cursor reconciliation, rigid linear stage enforcement, parallel doc validators) rather than leveraging stable integration contracts.

The objective of this architectural pivot is to redefine `ce-ai` v2 around clear, defensible boundaries:
1. **CE owns workflow semantics and artifacts.**
2. **Hosts own native plugin execution and packaging.**
3. **CE-AI coordinates the environment around them as a Cross-Host Operational Companion.**

## Key Learnings & Decisions

### 1. Shift from Authoritative FSM to Advisory Workflow Observation
- **What**: Deprecate storing the authoritative `current_stage` cursor in `~/.config/ce-ai/state.json`. Instead, `ce-ai workflow status` and `resume` observe progress dynamically from repository artifacts (`docs_root/plans/`, git branch name, commit history, open PR state, and run receipts).
- **Why**: Upstream CE considers git artifacts to *be* the workflow state. Storing state separately in `state.json` creates dual sources of truth and forces brittle heuristic reconciliation.
- **Where**: `src/commands/workflow.rs`, `src/state/state.rs`.
- **Learned**: CE is fundamentally a non-linear workflow graph (with skips, `ce-debug` loops, and conditional compounding), not a rigid 7-stage sequential pipeline.

### 2. Fleet Version Governance over File-Level Hashing
- **What**: Retire raw release tarball scraping and recursive SHA256 file diffing against host directories. Pivot `ce-ai fleet` to drive each host's native plugin manager (Claude Code marketplace, OpenCode extensions, Pi npm packages) and audit release version parity across the developer's toolchain.
- **Why**: Copying raw `skills/` directly onto disk bypasses host-specific packaging transforms (such as the Bun converter) and creates `external-duplicate` conflicts when running alongside native installations.
- **Where**: `src/source/archive.rs`, `src/state/diff.rs`, `src/commands/sync.rs`.
- **Learned**: Governed version alignment across multiple AI harnesses is high-value; duplicating package files locally is high-drag.

### 3. Upstream Compatibility Layer & Contract Boundaries
- **What**: Introduce a dedicated, typed Rust module `src/compat/` (`CeRelease`, `CeDocsConfig`, `CeSolutionFrontmatter`) that builds strictly on documented upstream contracts:
  - Stable skill names and documented mode tokens (`mode:return-to-caller`).
  - Dynamic `docs_root` loaded from `.compound-engineering/config.yaml`.
  - Upstream `schema.yaml` with explicit `schema_version`.
  - Native host install CLI commands.
- **Why**: Relying on internal layout details, prose, bundled references, or file hashes caused 7 concrete bugs in `ce-ai` v1.
- **Where**: `src/compat/`, `src/commands/doc.rs`.
- **Learned**: Isolate upstream knowledge into a typed boundary rather than sprinkling layout assumptions across command handlers.

## Migration Verification

- Validated via strict monotonic accretion of `CONCEPTS.md` (+5 entries added, 0 scrubbed).
- Formalized comprehensive PRD at `docs/architecture/ce-ai-v2-architecture-prd.md`.
- Documented OpenSpec change package at `openspec/changes/ce-ai-v2-architecture-prd/`.
- Verified 100% green pass on `ce-ai doc lint --strict`.
