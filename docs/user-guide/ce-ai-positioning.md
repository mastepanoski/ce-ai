<!-- Diátaxis Quadrant: Explanation | Audience: Beginner / Senior -->
# CE-AI, Compound Engineering, and ODD

CE-AI is an **engineering workflow for coding agents**. It does not try to replace the agent that writes code. Instead, it makes the surrounding engineering process explicit: what is being changed, what constraints apply, what has been verified, and what the next task can learn from this one.

## The boundary in one view

```text
Compound Engineering (EveryInc)
methodology and reusable skills (/ce-...)
          │
          ├── AI Coding Hosts (Claude Code, OpenCode, Cursor, Codex, Pi, ...)
          │   reason about tasks, execute tools, and manage native packaging
          │
          ├── Companion Developer Tools
          │   ├── CodeGraph: structural AST graph & blast-radius indexing
          │   ├── Engram: persistent SQLite+FTS5 memory across sessions
          │   └── RTK: CLI terminal output compression (60-90% token reduction)
          │
          └── CE-AI (Cross-Host Operational Companion)
              coordinates the operational environment around them
              ├── Fleet Version Governance: declarative multi-harness parity (ce-ai fleet)
              ├── Project Adoption & Health: non-destructive AGENTS.md rules & hooks (init-prj)
              ├── Agent Guardrails & Risk Engine: pre-execution command policy & safety (ce-ai guard)
              └── Advisory Workflow: 7-stage observation & ODD fast-path (workflow)
```

The layers have distinct jobs:

| Layer | Job | Examples |
| --- | --- | --- |
| **Coding agent / host** | Reasons about a task, edits code, and operates native toolchains. | Claude Code, Codex, OpenCode, Cursor, Pi |
| **Compound Engineering** | Supplies a methodology and multi-host skill set for compounding engineering work. | `ce-brainstorm`, `ce-plan`, `ce-work`, `ce-compound` |
| **Companion Tools** | Structural codebase index (CodeGraph), persistent memory (Engram), and token compression (RTK). | `codegraph init`, `mem_save`, `rtk` hooks |
| **CE-AI** | Coordinates the operational environment: fleet version governance, project adoption, companion verification, and agent guardrails. | `ce-ai fleet sync`, `init-prj`, `doctor`, `ce-ai guard` |

This is why CE-AI is not an alternative to Claude Code, Codex, OpenCode, or the official Compound Engineering plugin. It is the operational companion that helps those tools participate in a coherent, recoverable, and fleet-aligned engineering process.

## What Compound Engineering contributes

[Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin) is EveryInc’s methodology and official plugin for coding agents. Its central principle is simple: record useful outcomes so the next unit of work starts from a better position.

CE-AI uses that principle as its foundation. It does not claim to own the methodology, reproduce the official project, or substitute for its skill collection. CE-AI explores what happens when the operational environment around those skills is also made declarative and recoverable: multi-harness versions stay aligned, workflow state can be resumed, project artifacts can be checked, and completed work leaves durable evidence.

## Responsibilities, not a feature shootout

This is not a claim that one project has more features than another. The projects solve different parts of the same problem:

| Concern | Primary home | CE-AI’s relationship |
| --- | --- | --- |
| Reusable agent skills and host distribution | [EveryInc’s Compound Engineering plugin](https://github.com/EveryInc/compound-engineering-plugin) | CE-AI drives native host package managers to install and align them; it does not present a competing skill collection. |
| Agent reasoning, editing, and tool execution | The coding-agent host | CE-AI does not replace the agent or independently write code. |
| Multi-harness fleet version governance | CE-AI (`ce-ai fleet`) | CE-AI declaratively pins and synchronizes Compound Engineering versions across all developer hosts. |
| Project adoption, deterministic rules, and session hooks | CE-AI (`ce-ai init-prj`) | CE-AI injects non-destructive `AGENTS.md` rules and session-start recovery hooks into repositories. |
| Companion developer tools (memory, AST graph, token reduction) | Engram, CodeGraph, RTK | CE-AI configures hooks, MCP sidecars, and audits companion readiness via `doctor` and `tools status`. |
| Lightweight tactical execution | Compound Agile Fast-Path | CE-AI observes direct work and lightweight plans (`docs/plans/`), keeping small changes light without formal specs. |
```

As checked on 2026-09-24, the official plugin describes 36 skills across 14 hosts, including newer capabilities such as `ce-explain`, `ce-pov`, Compound Packs, and evolved multi-host support. CE-AI complements those capabilities by orchestrating the project-level workflow rather than duplicating upstream skills.

## The engineering loop

```text
Idea → Explore → Specify → Plan → Implement → Validate → Learn → Compound
```

The sequence is a mental model, not a demand for ceremony on every edit:

- **Idea and Explore** establish the outcome and examine the relevant context.
- **Specify and Plan** make meaningful scope, constraints, and execution order reviewable.
- **Implement and Validate** turn the plan into code and evidence.
- **Learn and Compound** preserve reusable conclusions so future work does not rediscover the same facts.

For substantial architectural work (> 200 LOC or cross-cutting contracts), CE-AI can coordinate optional OpenSpec change packages. For routine, tactical work, developers use the agile fast-path (`docs/plans/` or direct execution via `/ce-work` / `/ce-debug`).

## Agile Fast-Path: Bounded Tactical Execution

Originally adapted from [Alan Buscaglia's](https://github.com/Alan-TheGentleman) Organic Driven Development (ODD) in [Gentle AI](https://github.com/Gentleman-Programming/gentle-ai), CE-AI embraces the principle that everyday bugfixes, minor chores, and localized edits must never be burdened by mandatory specification ceremony.

In CE-AI, this fast-path is aligned natively with Compound Engineering:
- Tactical bugfixes run directly via `/ce-debug` or `/ce-work`.
- Lightly planned tasks maintain a single plan in `docs/plans/`.
- The artificial `odd/tasks/` directory and `workflow graduate` commands are deprecated in favor of native planning and direct OpenSpec adoption only when architectural complexity warrants it. See the [Agile Fast-Path Masterclass](odd-fast-path-and-graduation-masterclass.md) for details.
