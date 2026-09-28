# CE-AI — Compound Engineering Workflow Orchestration & Governance

**Open-source workflow orchestration and governance for Compound Engineering across AI coding agents.** Turn ideas into verified work, preserve engineering context, and make each change improve the next. CE-AI is not a coding agent, a harness replacement, or an agent runtime: Claude Code, Codex, Cursor, and OpenCode execute the probabilistic work; `ce-ai` governs the engineering workflow, state transitions, drift auditing, and multi-harness asset synchronization.

## Why CE-AI?

A developer familiar with [Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin) will ask: *why isn't the plugin itself enough?*

The Compound Engineering Plugin provides the skills and workflow methodology (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-compound`). `ce-ai` provides the operational and governance layer around them:

- **Multi-harness workflow integration**: Installs, synchronizes, and drift-audits plugin assets across 10 native harnesses (Claude Code, OpenCode, Cursor, Codex, Copilot, AGY, Kimi, Grok, Pi, FX) with atomic writes and per-harness pre-mutation backups.
- **Workflow FSM & stage tracking**: Records and validates legal transitions across 7 development stages; teams supply the test and knowledge-capture evidence required by their workflow.
- **Checkpoints & drift recovery**: Snapshots progress before context compactions and synchronizes disk reality (`RepoState`) upon resumption.
- **Project adoption**: Injects tamper-evident, SHA256-verified workflow contracts into project rule files (`ce-ai init-prj`).
- **Skill registry & model profiles**: Discovers skills across hosts, manages workspace isolation, and handles role-scoped model assignments.

> **Key principle**: Workflow bookkeeping and state transitions can be deterministic; agent execution remains probabilistic. `ce-ai` governs the state and contracts so probabilistic agent work stays verifiable.

## How the pieces fit

```text
Compound Engineering (EveryInc)
        │
        │ skills + engineering workflow
        ▼
      ce-ai
 workflow orchestration + governance
        │
        ▼
 AI coding harnesses
 Claude Code / Codex / Cursor / OpenCode / Copilot / AGY / …
```

CE-AI complements the official [Compound Engineering plugin](https://github.com/EveryInc/compound-engineering-plugin); it does not replace it or compete with coding agents. See [CE-AI positioning](docs/user-guide/ce-ai-positioning.md) for the full architectural breakdown.

## Try it in two minutes

> Prerequisite: install one [supported coding-agent host](docs/user-guide/harness-matrix.md) first.

```bash
# 1. Install companion tools (persistent memory, codebase graph, token reduction)
brew install gentleman-programming/tap/engram rtk # or go/cargo; Windows: see Getting Started
curl -fsSL https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh | sh

# 2. Install ce-ai (Homebrew, or: curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash)
brew install mastepanoski/ce-ai/ce-ai

# 3. In your project: initialize graph, install plugin & sidecars, adopt, and verify
cd your-project && codegraph init
ce-ai install --harness all && ce-ai init-prj && ce-ai doctor
```

Then reopen your coding agent in `your-project` and start with `/ce-brainstorm <outcome>`. For a guided first run, follow [Getting Started](docs/user-guide/getting-started.md).

## The 7-stage workflow & FSM

```text
Ideation → OpenSpec → Plan → Work/TDD → Verify → Compound → Ship
```

`ce-ai` governs progression through this flywheel by validating stage transitions and surfacing repository state. Teams attach empirical verification and capture decisions in `docs/solutions/`; for formal changes it coordinates [OpenSpec](openspec/specs/), while routine tactical tasks can use Organic Driven Development (ODD, by Alan Buscaglia / Gentle AI).

## Documentation map

| Document | Audience | Intent |
| --- | --- | --- |
| 🌱 [Getting Started](docs/user-guide/getting-started.md) | Beginner | Tutorial — install, adopt a project, and run a first workflow step |
| 🎓 [CE-AI positioning](docs/user-guide/ce-ai-positioning.md) | Beginner / Senior | Explanation — agents, Compound Engineering, ODD, and CE-AI’s boundary |
| 🚀 [Quick Start Workflow Guide](docs/user-guide/quick-start-workflow-guide.md) | Beginner | Tutorial — first feature, bug fix, and workflow resumption |
| 🎓 [Compound Workflow Explained](docs/user-guide/compound-engineering-workflow-explained.md) | Beginner | Explanation — how strategy becomes code and durable learning |
| 🎓 [Documentation Debt & Hygiene](docs/user-guide/doc-hygiene-and-debt-explained.md) | Beginner | Explanation — keep specifications and guidance current |
| 📁 [Project Adoption Guide](docs/user-guide/project-adoption-guide.md) | Both | How-to — safely adopt or de-adopt a project |
| 🎓 [Harnesses, Loops & Context](docs/user-guide/harnesses-loops-and-context-masterclass.md) | Beginner | Explanation — coding-agent hosts, context, and supporting tools |
| 🎓 [Determinism Explained](docs/user-guide/determinism-explained.md) | Beginner | Explanation — what CE-AI can and cannot guarantee |
| 🎓 [Zero-Step Drift Recovery](docs/user-guide/zero-step-drift-recovery-explained.md) | Beginner | Explanation — recover current repository state at session start |
| ⚡ [ODD Fast-Path Masterclass](docs/user-guide/odd-fast-path-and-graduation-masterclass.md) | Beginner / Senior | Explanation — the lightweight path and when to graduate to formal artifacts |
| 🧠 [Decision Engine Guide](docs/user-guide/decision-engine-guide.md) | Both | How-to / Reference — routing, risk checks, and readiness |
| 🔧 [Installation & Coexistence](docs/user-guide/installation-and-coexistence-mechanisms.md) | Both | How-to — install without overwriting existing configurations |
| 🔄 [Sync & Upgrade](docs/user-guide/sync-and-upgrade-mechanisms.md) | Both | How-to — reconcile drift, update, and roll back |
| ⚡ [Skill Registry Guide](docs/user-guide/skill-registry-guide.md) | Both | How-to / Reference — find and resolve available skills |
| 💾 [Backup & Uninstall](docs/user-guide/backup-and-uninstall.md) | Both | How-to — restore configurations and remove CE-AI safely |
| 🗂️ [Harness Matrix](docs/user-guide/harness-matrix.md) | Senior | Reference — supported hosts, configuration paths, and integration methods |
| 🏛️ [Architecture Guide](docs/user-guide/architectural-and-conceptual-guide.md) | Senior | Explanation — deterministic state, adapters, and project artifacts |
| 🎮 [FSM & Checkpoints](docs/user-guide/fsm-and-checkpoints-explained.md) · [vs. Memory](docs/user-guide/checkpoints-vs-memory-explained.md) | Both | Explanation — lifecycle stages, checkpoints, and session memory |
| 🎓 [Spec-Driven vs. Chat TDD](docs/user-guide/spec-driven-vs-chat-tdd-explained.md) | Senior | Explanation — context economics, workflow granularity, and bounded TDD |
| 📐 [OpenSpec specifications](openspec/specs/) | Senior | Reference — living system specifications and contracts |
| 🧠 [Solutions Library](docs/solutions/) · [Plans & Audits](docs/plans/) | Contributor | Reference — solved problems, decisions, and delivery history |

## Acknowledgments

CE-AI builds on [EveryInc’s Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin). [Organic Driven Development (ODD)](https://github.com/Gentleman-Programming/gentle-ai/blob/main/docs/intended-usage.md) was created by [Alan Buscaglia](https://github.com/Alan-TheGentleman) through [Gentle AI](https://github.com/Gentleman-Programming/gentle-ai); CE-AI adapts it as an optional workflow path. It also draws on [Engram](https://github.com/Gentleman-Programming/engram), [CodeGraph](https://github.com/colbymchenry/codegraph), [Context7](https://github.com/upstash/context7), [RTK](https://github.com/rtk-ai/rtk), and [Sequential Thinking](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking).

## Project links

[Security](SECURITY.md) · [AI policy](AI_POLICY.md) · [Contributing](CONTRIBUTING.md) · [Documentation style](docs/references/docs-styling.md) · [MIT License](LICENSE)
