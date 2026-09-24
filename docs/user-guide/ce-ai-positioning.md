<!-- Diátaxis Quadrant: Explanation | Audience: Beginner / Senior -->
# CE-AI, Compound Engineering, and ODD

CE-AI is an **engineering workflow for coding agents**. It does not try to replace the agent that writes code. Instead, it makes the surrounding engineering process explicit: what is being changed, what constraints apply, what has been verified, and what the next task can learn from this one.

## The boundary in one view

```text
Compound Engineering
methodology and reusable skills
          │
          ├── Official Compound Engineering plugin
          │   distributes skills to coding-agent hosts
          │
          └── CE-AI
              coordinates the engineering workflow in a project
              │
              └── agents, LLMs, tools, project artifacts, and verification
```

The layers have distinct jobs:

| Layer | Job | Examples |
| --- | --- | --- |
| **Coding agent / host** | Reasons about a task and operates tools. | Claude Code, Codex, OpenCode, Cursor |
| **Compound Engineering** | Supplies a methodology and multi-host skill set for compounding engineering work. | `ce-brainstorm`, `ce-plan`, `ce-work`, `ce-compound` |
| **CE-AI** | Makes the workflow durable inside a project through adoption, workflow state, artifacts, health checks, and adaptive routing. | `ce-ai init-prj`, `workflow resume`, `doctor` |

This is why CE-AI is not an alternative to Claude Code, Codex, OpenCode, or the official Compound Engineering plugin. It is the layer that helps those tools participate in a coherent, recoverable engineering process.

## What Compound Engineering contributes

[Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin) is EveryInc’s methodology and official plugin for coding agents. Its central principle is simple: record useful outcomes so the next unit of work starts from a better position.

CE-AI uses that principle as its foundation. It does not claim to own the methodology, reproduce the official project, or substitute for its skill collection. CE-AI explores what happens when the process around those skills is also made executable at the project level: workflow state can be resumed, project artifacts can be checked, and completed work can leave durable evidence.

## Responsibilities, not a feature shootout

This is not a claim that one project has more features than another. The projects solve different parts of the same problem:

| Concern | Primary home | CE-AI’s relationship |
| --- | --- | --- |
| Reusable agent skills and host distribution | [EveryInc’s Compound Engineering plugin](https://github.com/EveryInc/compound-engineering-plugin) | CE-AI installs and coordinates the workflow around those skills; it does not present a competing skill collection. |
| Agent reasoning, editing, and tool execution | The coding-agent host | CE-AI does not replace the agent or independently write code. |
| Project adoption, deterministic workflow state, health/drift checks, and artifact visibility | CE-AI | CE-AI makes the process surrounding agent work inspectable and recoverable in a repository. |
| Lightweight everyday routing | [Gentle AI’s ODD](https://github.com/Gentleman-Programming/gentle-ai/blob/main/docs/intended-usage.md) | CE-AI adapts the approach as an optional fast path and credits its origin. |

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

For substantial work, CE-AI can coordinate formal OpenSpec artifacts. For routine, understood work, it can use ODD instead.

## ODD: the adaptive path

**Organic Driven Development (ODD)** was created by [Alan Buscaglia](https://github.com/Alan-TheGentleman) through [Gentle AI](https://github.com/Gentleman-Programming/gentle-ai). Gentle AI documents it as the everyday path: keep small, understood changes lightweight; retain one recoverable task record when work is substantial; select formal Spec-Driven Development only when it is wanted.

CE-AI does not claim ODD as its own. It adapts ODD as an optional fast path for bounded, tactical work. Its adaptive mode router can keep a small fix light, while its graduation bridge moves work into formal OpenSpec artifacts when the task becomes broader or more consequential. See the [ODD Fast-Path Masterclass](odd-fast-path-and-graduation-masterclass.md) for CE-AI’s implementation details.
