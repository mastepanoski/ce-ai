<!-- Diátaxis Quadrant: Explanation | Audience: Beginner / Senior -->
# 🎓 Masterclass: Harnesses, Agent Loops & Context Engineering

Welcome to the **Harnesses, Agent Loops & Context Engineering Masterclass**! This guide is designed specifically for newcomers to explain how AI editors (Harnesses), autonomous execution cycles (Agent Loops), and context optimization mechanics work together inside **`ce-ai`** and **Compound Engineering**.

---

## 1. What is an AI Harness?

### 🧩 Everyday Analogy: The Car Engine vs. The Steering Wheel
Imagine an AI model (like Gemini, Claude, or GPT) is a powerful **racing engine**. 

An **AI Harness** is the **vehicle body, dashboard, and steering wheel** surrounding that engine. It provides the user interface, file-system access, terminal execution, and plugin systems that allow the AI engine to interact with your codebase.

```mermaid
flowchart TD
    ENGINE["AI Model Engine (Gemini / Claude / GPT)"] --> HARNESS["AI Harness (Editor / Environment)"]
    HARNESS --> TOOLS["FileSystem, Stdio Terminal, Plugin Systems"]
    TOOLS --> CODEBASE["Your Local Workspace & Git Repository"]
```

### Supported Harnesses in `ce-ai`:
`ce-ai` is an **engineering workflow for coding agents** across multiple hosts. It detects, configures, and synchronizes Compound Engineering integrations across:

| Harness Name | Environment Type | How `ce-ai` Integrates |
| :--- | :--- | :--- |
| **OpenCode** | Open-source CLI Harness | Manages `~/.config/opencode/opencode.json` & skill arrays |
| **Claude Code** | Anthropic Terminal Harness | Configures `~/.claude.json` & managed plugin directories |
| **Cursor** | VSCode-based AI Editor | Injects managed rules into `.cursorrules` / `.cursor/rules/` |
| **GitHub Copilot** | JetBrains & VSCode Extension | Manages instructions in `.github/copilot-instructions.md` |
| **Antigravity CLI (`agy`)** | Autonomous Agentic CLI | Native integration via shared `state.json` & sidecars |
| **Pi / Custom JSON** | Custom/Experimental Harnesses | Implements `HarnessAdapter` trait for custom JSON files |

---

## 2. MCP Sidecars vs. CLI Token Reducers (RTK)

A common point of confusion for beginners is the difference between **MCP Sidecars** and **CLI Token Reducers**.

```mermaid
flowchart LR
    AGENT["AI Agent"] -->|JSON-RPC Protocol| MCP["MCP Sidecars (Engram / CodeGraph)"]
    AGENT -->|Shell Command Execution| RTK["RTK Token Reducer (Terminal Pre-Processor)"]
    RTK -->|Filter & Compress Stdio Output| SHELL["Raw Stdio Output (git, cargo, docker)"]
```

### 1. MCP Sidecars (Protocol-Based Intelligence)
- **What they are**: Model Context Protocol (MCP) servers that run in the background over `stdio` / `JSON-RPC`.
- **Examples in `ce-ai`**:
  - **Engram**: Persistent cross-session memory server. Stores architecture decisions, bug fixes, and user preferences across sessions.
  - **CodeGraph**: Call-graph and blast-radius indexing server. Analyzes functions, callers, callees, and dependencies.
- **Role**: They give the agent **deep long-term memory** and **structural codebase intelligence**.

### 2. CLI Token Reducers (Terminal Output Filters)
- **What they are**: Terminal pre-processors (such as **RTK / Rust Token Killer**) that intercept shell command outputs before sending text to the LLM.
- **Example**: Running `cargo test` or `docker ps` can generate 5,000 lines of verbose terminal output. RTK filters noise (whitespace, duplicate warnings, passing test boilerplate) and compresses the stream by **60% to 90%**.
- **Role**: They save token costs and prevent context window exhaustion. **RTK is NOT an MCP server**; it is a CLI output filter!

---

## 3. What is an Agent Loop?

### 🔄 The Execution Cycle: Read-Evaluate-Act-Reflect (REAR)
An **Agent Loop** is the autonomous, iterative cycle an AI agent runs when executing a prompt:

```mermaid
flowchart TD
    P[User Prompt] --> READ[1. Read Codebase & State]
    READ --> EVAL[2. Evaluate Hypothesis & Plan]
    EVAL --> ACT[3. Act: Edit File or Run Terminal Command]
    ACT --> REFLECT[4. Reflect on Test Results & Output]
    REFLECT -->|Not Done| READ
    REFLECT -->|Task Verified| DONE[5. Deliver Output & Compound Knowledge]
```

### Key Agent Loops in Compound Engineering:

1. **The TDD Feedback Loop (Red-Green-Refactor)**:
   - *Red*: Agent writes a failing unit test reproducing the requirement or bug.
   - *Green*: Agent writes the minimal code implementation to pass the test.
   - *Refactor*: Agent runs `/ce-simplify-code` to tidy the implementation without altering behavior.

2. **The Diagnostic Loop (`ce-debug`)**:
   - Freezes forward FSM progress upon encountering a crash or test failure.
   - Formulates hypotheses ➔ extracts un-truncated logs ➔ writes minimal reproducer ➔ applies root cause fix ➔ verifies green test.

3. **The Compounding Knowledge Loop (`ce-compound`)**:
   - Executes at the end of every completed task (Stage 6).
   - Extracts hard-earned learnings, gotchas, and architectural decisions, storing them in `docs/solutions/` and `CONCEPTS.md`.
   - In future sessions, agent loops query these solution docs, preventing past mistakes from ever repeating!

---

## 4. Context Engineering & Token Economics

### 📉 The Problem: Context Compaction & Decay
LLMs have a finite **Context Window** (e.g. 128k, 200k, or 1M tokens). As an agent executes tool calls, views files, and runs terminal commands, earlier conversation turns are truncated or compressed via **Context Compaction**.

If an agent loses context halfway through a 10-step implementation plan, it may hallucinate missing details or repeat steps.

```mermaid
flowchart TD
    FULL_CONTEXT["100% Context Window\n(Fresh Session)"] --> TOOL_CALLS["Multiple File Views & Stdio Commands"]
    TOOL_CALLS --> COMPACTION["Context Compaction / Decay\n(Loss of Early Conversation)"]
    COMPACTION --> SOLUTION["ce-ai workflow checkpoint & Engram\n(100% Context Restoration on Disk)"]
```

### 🛡️ How `ce-ai` Solves Context Compaction:

1. **Atomic Workflow Checkpoints & Turn-0 Resumption (`ce-ai workflow checkpoint` / `resume`)**:
   - Saves current FSM stage index, active subtask string, and timestamp to disk (`state.json`).
   - On session start or context compaction, `ce-ai workflow resume` re-hydrates canonical Git branch, working tree state, manifest integrity, and OpenSpec progress.
   - Delivered automatically via native lifecycle hooks in Claude Code (`.claude/settings.json` `SessionStart`), OpenCode (`session.created` / `session.idle` event subscriptions + `context`/`compaction` system-context hooks in the `.opencode/plugins/compound-engineering.js` V2 plugin), GitHub Copilot CLI (`.github/hooks/hooks.json` `sessionStart`), OpenAI Codex CLI (`.codex/config.toml` `SessionStart`), Pi (`.pi/extensions/compound-engineering.ts` `before_agent_start`), Cursor (`.cursor/hooks.json` `sessionStart`), and Google Antigravity CLI (`.agents/hooks.json` `PreInvocation`, deduplicated per session). Kimi, Grok, DeepSeek, and Fx have no native context-injection hook (see [Zero-Step Drift Recovery Explained](zero-step-drift-recovery-explained.md#4-delivery-architecture-automated-hooks-vs-prompt-directives)) and remain governed by the Turn-0 prompt directive in `AGENTS.md`.

2. **Engram Memory Persistence**:
   - Session summaries and technical findings are saved outside the LLM context in Engram's SQLite database.
   - Agents query memory via `mem_context` or `mem_search`, instantly recalling past solutions regardless of token limits.

3. **CLI Token Reduction via RTK**:
   - Intercepts verbose terminal outputs (`cargo test`, `git status`, `docker ps`), stripping noise and preserving context capacity.

---

## 5. Ecosystem Credits and Boundaries

CE-AI combines independent projects with different roles. [Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin) is EveryInc’s official methodology and skill plugin; it is not a Gentle AI project. [Organic Driven Development (ODD)](https://github.com/Gentleman-Programming/gentle-ai/blob/main/docs/intended-usage.md) was created by [Alan Buscaglia](https://github.com/Alan-TheGentleman) through Gentle AI. CE-AI adapts ODD as an optional path and uses Compound Engineering as its methodological foundation.

```mermaid
flowchart TD
    CE_AI["CE-AI: engineering workflow for coding agents"] --> CE["EveryInc Compound Engineering
methodology and skills"]
    CE_AI --> ODD["Alan Buscaglia / Gentle AI
Organic Driven Development"]
    CE_AI --> ENGRAM["Engram
Persistent memory"]
    CE_AI --> CODEGRAPH["CodeGraph
Codebase intelligence"]
    CE_AI --> CONTEXT7["Context7
Library documentation"]
    CE_AI --> RTK["RTK
Terminal-output reduction"]
    CE_AI --> SEQ["Sequential Thinking
Structured reasoning"]
```

### Key ecosystem tools

1. **[Compound Engineering](https://github.com/EveryInc/compound-engineering-plugin)**: EveryInc’s methodology and multi-host skill plugin. CE-AI complements it with project-level workflow state and artifacts; it does not replace it.
2. **[Gentle AI](https://github.com/Gentleman-Programming/gentle-ai)**: The home of ODD, created by Alan Buscaglia. ODD informs CE-AI’s lightweight, recoverable execution path.
3. **[Engram](https://github.com/Gentleman-Programming/engram)**: Persistent memory sidecar powered by SQLite and FTS5 for decisions, discoveries, bug fixes, and preferences across sessions.
4. **[CodeGraph](https://github.com/colbymchenry/codegraph)**: Codebase intelligence for AST symbols, callers, callees, and blast-radius analysis.
5. **[Context7](https://github.com/upstash/context7)**: Current library and framework documentation retrieval.
6. **[RTK](https://github.com/rtk-ai/rtk)**: Terminal-output reduction for preserving context capacity.
7. **[Sequential Thinking](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking)**: Structured reasoning support for hypotheses and refinement.

---

## 👨‍🏫 Teacher's Guide: Understanding Proactive Workflow Observability (v0.6.0)

Think of an AI coding agent as a pilot flying a high-performance jet plane across complex software projects:

1. **The Workflow Status Engine (`ce-ai workflow status`)**:
   - *The Cockpit Instrument Panel*: Imagine flying blind without gauges showing speed, altitude, or fuel. The workflow status engine acts like your live cockpit display: it visualizes exactly which of the 7 Flywheel stages your AI agent is navigating, what subtask is active, and shows historical progress checkpoints saved to disk.

2. **Real-Time Sync Watcher (`ce-ai sync --watch`)**:
   - *The Automatic Autopilot Guardrail*: When multiple developers or harness tools edit local skills or configurations, files can drift out of sync. The `--watch` flag acts like an autopilot guardrail—continuously monitoring managed configuration paths in the background and re-syncing SHA256 integrity instantly upon detecting changes.

3. **Workspace Configuration Overrides (`.ce-ai.json`)**:
   - *Local Cockpit Presets vs Master Flight Plan*: Just like a pilot adjusting seat height or radio frequencies for a specific flight without changing standard airline defaults, `.ce-ai.json` allows team members to override model assignments (`ce-work`, `ce-plan`) locally per repository while preserving global developer preferences (`~/.config/ce-ai/state.json`).

---

## 📋 Masterclass Summary Checklist for Beginners

- [x] **Harness**: The AI editor/environment (Claude Code, Cursor, Copilot, Antigravity, OpenCode).
- [x] **MCP Sidecars**: Protocol-based background servers for memory (Engram), codebase graphs (CodeGraph), and docs (Context7).
- [x] **CLI Reducers**: Shell output filters (RTK) that shrink terminal output by 60–90%.
- [x] **Sequential Thinking**: Structured reasoning protocol for step-by-step problem decomposition.
- [x] **Agent Loop**: The autonomous Read-Evaluate-Act-Reflect cycle driven by TDD and verification.
- [x] **Context Engineering**: Using FSM checkpoints and Engram memory to overcome token compaction.
