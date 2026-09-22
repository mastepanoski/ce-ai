---
module: workflow
tags: [workflow-resume, openspec, archival, turn-0, doctor, status, block-version, v5]
problem_type: architecture
title: "Automated Post-Merge OpenSpec Archival & Turn-0 Prescriptions"
applies_when: "When AI agents conclude PR merges without archiving completed OpenSpec packages, leaving lingering workflow drift."
---

# Automated Post-Merge OpenSpec Archival & Turn-0 Prescriptions

## Problem
Across the `ce-ai` ecosystem, AI agents (Claude Code, OpenAI Codex, OpenCode, Gemini, Cursor) frequently merged Pull Requests on `main`, switched branches, and concluded their sessions without running `ce-ai archive <feature>`. This left completed OpenSpec change packages lingering indefinitely in `openspec/changes/`, causing:
1. `ce-ai workflow status` and `ce-ai doctor` to report persistent unarchived warnings.
2. Workflow FSM state on `main` to remain degraded instead of reaching `✓ Ready (100%)`.
3. Subsequent sessions and human operators to encounter confusing residual context.

Root cause analysis revealed three architectural defects:
- **Governance Blindspot**: Invariant #10 in `AGENTS.md` and adoption blocks strictly defined post-merge cleanup as git branch and remote pruning (`git pull`, `git branch -d`, `git fetch --prune`), completely omitting workflow verification and archival requirements.
- **Mode-Gated Resume Check**: In `src/commands/workflow.rs`, the `unarchived_completed_changes` check was guarded by `if mode == ExecutionMode::Compound`. On `main` without an active feature branch, execution mode defaulted to `Organic`, silently skipping the check.
- **Passive Diagnostic Pointers**: Both `ce-ai doctor` and `workflow status` emitted passive pointers to `README.md` or `run 'ce-ai doctor' for details` instead of providing the exact runnable command.

## Solution

Implemented a 4-tier closed-loop prescription architecture:

1. **Turn-0 Action Required Prescription (`src/commands/workflow.rs`):**
   - Decoupled `unarchived_completed_changes` detection from `ExecutionMode::Compound`.
   - When completed unarchived change packages exist in `openspec/changes/`, `resume_lines_with_mode` emits an explicit, high-visibility action directive across all execution modes:
     ```text
     openspec ledger: ! 1 change(s) complete but not archived — run 'ce-ai archive <feature>'
     ! Action Required: OpenSpec change '<feature>' is complete (<completed>/<total> tasks). Run 'ce-ai archive <feature>' to seal the change package.
     ```
   - Automatically delivered to agents via session-start native hooks (`SessionStart` / `PreInvocation`) and CLI invocation.

2. **Actionable CLI Diagnostics (`src/commands/doctor.rs`, `src/commands/workflow.rs`):**
   - Updated `ce-ai doctor` finding string to provide the exact runnable command:
     `doctor-warn: openspec change '<feature>' is complete (N/N tasks) but not archived — run 'ce-ai archive <feature>'`
   - Updated `ce-ai workflow status` warning line to:
     `! Warning: N OpenSpec change(s) complete but not archived — run 'ce-ai archive <feature>'`

3. **Managed Adoption Block Evolution (`BLOCK_VERSION: 5` in `src/commands/init_prj.rs`):**
   - Bumped `BLOCK_VERSION` 4 $\to$ 5 in `src/commands/init_prj.rs` and `tests/cli.rs`.
   - Added `### 🧹 Post-Merge Lifecycle & Clean State` governance to Full-tier and Orchestrator-tier managed templates, mandating that agents run `ce-ai workflow status` and `ce-ai archive <feature>` post-merge before session conclusion.

4. **Hard Invariant #10 Governance Update (`AGENTS.md`):**
   - Updated Hard Invariant #10 to `Post-Merge Lifecycle & Clean State`, instructing agents to verify `ce-ai workflow status` and archive completed OpenSpec changes before session conclusion.

## Key Learnings

1. **Imperative Directives Outperform Passive Pointers**: AI agents operate deterministically on direct command prompts. Changing diagnostic messages from passive documentation pointers (`see openspec/changes/archive/README.md`) to explicit imperative commands (`run 'ce-ai archive <feature>'`) and prominent action directives (`! Action Required: ...`) ensures agents execute the remediation autonomously.
2. **Organic Mode Must Surface Ledger Health**: While active feature development takes place on feature branches in `Compound` mode, post-merge verification occurs on `main` where mode defaults to `Organic`. Diagnostic ledger checks must evaluate across all modes to prevent silent gaps after merging.
3. **Coordinated Block Version Bumping**: When updating managed block templates in `init_prj.rs`, bumping `BLOCK_VERSION` ensures older adoptions are classified as `StaleVersion` with an actionable upgrade hint rather than misleading `DriftDetected`. All test fixtures utilizing `CUR_BLOCK_VERSION` in `tests/cli.rs` must be kept in lockstep.
