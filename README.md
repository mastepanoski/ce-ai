# CE-AI — The engineering workflow for coding agents

**CE-AI is not another coding agent. It is an engineering workflow for coding agents.** It gives Claude Code, Codex, OpenCode, Cursor, and other supported hosts a durable way to turn an idea into verified work—and make each completed change improve the next one.

## The 30-second version

| Question | Answer |
| --- | --- |
| **What is CE-AI?** | An open-source engineering workflow for coding agents that coordinates work, verification, and durable learning around the agent you already use. |
| **Why does it exist?** | Agents can write code quickly; CE-AI makes scope, specifications, verification, and learned context visible and recoverable across sessions. |
| **How is it different?** | Claude Code, Codex, and OpenCode are **agents/hosts**. Compound Engineering is the **methodology and skill plugin**. CE-AI is the **engineering workflow for coding agents** around them. |
| **What is Compound Engineering?** | EveryInc’s methodology for making each unit of engineering work easier than the last through reusable skills and durable learning. |
| **What is ODD?** | Organic Driven Development: Alan Buscaglia’s Gentle AI approach for keeping understood work lightweight while retaining recoverable context. CE-AI adapts it as an optional fast path. |

```text
Idea → Explore → Specify → Plan → Implement → Validate → Learn → Compound
```

“Compound” means that verified decisions and lessons become inputs for future work—not a forgotten chat transcript.

## How the pieces fit

```text
Coding agents and hosts (Claude Code, Codex, OpenCode, Cursor, …)
                              │ execute work
Compound Engineering ─────────┤ supplies methodology and reusable skills
                              │
CE-AI ────────────────────────┘ makes the workflow inspectable, adaptive, and durable in a project
```

CE-AI complements the official [Compound Engineering plugin](https://github.com/EveryInc/compound-engineering-plugin); it does not replace it or compete with coding agents. See [CE-AI positioning](docs/user-guide/ce-ai-positioning.md) for the full explanation.

## Try it in two minutes

> Prerequisite: install one [supported coding-agent host](docs/user-guide/harness-matrix.md) first.

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.sh | bash

# Windows PowerShell
irm https://raw.githubusercontent.com/mastepanoski/ce-ai/main/scripts/install.ps1 | iex

# Homebrew (macOS / Linux)
brew install mastepanoski/ce-ai/ce-ai

# From source
cargo install --path .

# Then, in your project
cd your-project && ce-ai install --harness all && ce-ai init-prj && ce-ai doctor
```

Then reopen your coding agent in `your-project` and start with `/ce-brainstorm <outcome>`. For a guided first run, follow [Getting Started](docs/user-guide/getting-started.md).

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
| 🎮 [FSM & Checkpoints](docs/user-guide/fsm-and-checkpoints-explained.md) | Senior | Explanation — lifecycle stages and checkpoints |
| ⚖️ [Checkpoints vs. Memory](docs/user-guide/checkpoints-vs-memory-explained.md) | Beginner | Explanation — why checkpoints differ from session notes |
| 🧭 [Workflow Panel](docs/user-guide/workflow-panel-native-vs-agent-skills.md) | Beginner | Explanation — native dashboard actions versus agent skills |
| 📐 [OpenSpec specifications](openspec/specs/) | Senior | Reference — living system specifications and contracts |
| 🧠 [Solutions Library](docs/solutions/) · [Plans & Audits](docs/plans/) | Contributor | Reference — solved problems, decisions, and delivery history |

## Acknowledgments

CE-AI builds on [EveryInc’s Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin). [Organic Driven Development (ODD)](https://github.com/Gentleman-Programming/gentle-ai/blob/main/docs/intended-usage.md) was created by [Alan Buscaglia](https://github.com/Alan-TheGentleman) through [Gentle AI](https://github.com/Gentleman-Programming/gentle-ai); CE-AI adapts it as an optional workflow path. It also draws on [Engram](https://github.com/Gentleman-Programming/engram), [CodeGraph](https://github.com/colbymchenry/codegraph), [Context7](https://github.com/upstash/context7), [RTK](https://github.com/rtk-ai/rtk), and [Sequential Thinking](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking).

## Project links

[Security](SECURITY.md) · [AI policy](AI_POLICY.md) · [Contributing](CONTRIBUTING.md) · [Documentation style](docs/references/docs-styling.md) · [MIT License](LICENSE)
