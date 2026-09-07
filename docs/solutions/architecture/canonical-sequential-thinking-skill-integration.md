---
title: "Canonical Sequential-Thinking Skill Integration & Zero-Daemon Cognitive Invariant"
category: "architecture"
date: "2026-09-06"
tags:
  - skills
  - sequential-thinking
  - cognitive-invariants
  - no-mcp
  - pi
  - opencode
  - custom
  - registry
  - doctor
components:
  - source::builtin_skills
  - source::registry
  - source::tools_registry
  - commands::install
  - commands::sync
  - commands::doctor
applies_when: "Resolving reasoning skills, resolving sequential-thinking, diagnosing unconfigured companion suggestions, or enforcing No-MCP harness parity"
---

# Canonical Sequential-Thinking Skill Integration & Zero-Daemon Cognitive Invariant

## Context

Prior to Issue #309, `sequential-thinking` was defined as an on-demand skill suggestion in `src/source/tools_registry.rs` (`resolve_cmd: "ce-ai skills resolve sequential-thinking"`). However, executing that command resulted in:

```markdown
<!-- ce-ai:skill_resolution status=none -->
## Skills to load before work:
```

Because no physical `sequential-thinking/SKILL.md` existed on disk or in the upstream plugin tarball, `SkillRegistry::build` never indexed it. Consequently:
1. `is_skill_configured("sequential-thinking")` evaluated to `false`.
2. `ce-ai doctor` and `tools status` perpetually warned that `sequential-thinking` was unconfigured.
3. The suggested resolution command yielded an empty list without actionable guidance.

## Decision: Skill Protocol vs. MCP Server

Issue #309 evaluated two architectural options:
- **Option (a) - Registered MCP Server**: Required spawning an external Node.js daemon running `@modelcontextprotocol/server-sequential-thinking`. Rejected due to heavy process footprint, IPC round-trip overhead, and violation of the strict No-MCP architecture of the `Pi` harness.
- **Option (b) - On-Demand Native Skill**: Adopted. Disperses structured chain-of-thought discipline directly in the LLM's context window with zero runtime dependencies.

## Solution Architecture

Release v1.43.0 closes the gap with a canonical, compile-time embedded skill asset and automated seeding:

### 1. Canonical Skill Protocol (`skills/sequential-thinking/SKILL.md`)
- Authoritative definition of step-by-step reasoning: step progression, hypothesis testing, dynamic revision, falsification criteria, and final convergence.
- Unified frontmatter supporting both `ce-ai`'s `SkillRegistry` (`name`, `description`, `scope`, `triggers`) and host harness slash-commands (`argument-hint`).

### 2. Compile-Time Embedded Asset (`src/source/builtin_skills.rs`)
- `BUILTIN_SEQUENTIAL_THINKING_SKILL`: Embedded via `include_str!("../../skills/sequential-thinking/SKILL.md")`.
- `seed_builtin_skill` / `seed_custom_builtin_skill`: Atomic seeding helpers ensuring files are created cleanly behind `write_atomic` and gated by `!dry_run`.

### 3. Lifecycle Fallback Seeding (`install` and `sync`)
- When `ce-ai install` or `ce-ai sync` runs, `managed_tree` prioritizes source files if present.
- If absent from the source root (e.g. upstream plugin tarball lag), the embedded skill is automatically seeded into `<config_dir>/skills/` and harness-specific managed directories.
- Dry-run safety (SU-4) is strictly respected: planned actions are logged without performing writes.

### 4. Deterministic Indexing & Auto-Configured Doctor
- `SkillRegistry::build` indexes `sequential-thinking` with SHA256 integrity hashing and maps paths across all harnesses.
- `ce-ai skills resolve sequential-thinking` produces `status=paths-injected` with a verified `file://` URI.
- `is_skill_configured(&ctx, "sequential-thinking")` automatically evaluates to `true` upon reading `skills-registry.json`, silencing diagnostic warnings without ad-hoc code in `doctor.rs` or `tools_registry.rs`.
