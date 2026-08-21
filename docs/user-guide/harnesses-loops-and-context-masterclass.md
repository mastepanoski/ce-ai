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
`ce-ai` is a **Multi-Harness Orchestrator**. It detects, configures, and synchronizes Compound Engineering plugins across:

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

1. **Atomic Workflow Checkpoints (`ce-ai workflow checkpoint`)**:
   - Saves current FSM stage index, active subtask string, and timestamp to disk (`state.json`).
   - If context compacts or a session ends, running `ce-ai workflow resume` re-hydrates the exact state from disk.

2. **Engram Memory Persistence**:
   - Session summaries and technical findings are saved outside the LLM context in Engram's SQLite database.
   - Agents query memory via `mem_context` or `mem_search`, instantly recalling past solutions regardless of token limits.

3. **CLI Token Reduction via RTK**:
   - Intercepts verbose terminal outputs (`cargo test`, `git status`, `docker ps`), stripping noise and preserving context capacity.

---

## 5. Ecosystem Acknowledgments & Inspiration

`ce-ai` and **Compound Engineering** build upon pioneering open-source projects in the **`gentle-ai`** ecosystem:

```mermaid
flowchart TD
    GENTLE["gentle-ai Ecosystem (Foundational Inspiration)"] --> CE_AI["ce-ai & Compound Engineering"]
    CE_AI --> ENGRAM["Engram (Persistent Memory Sidecar)"]
    CE_AI --> CODEGRAPH["CodeGraph (Blast-Radius & Call-Graph Indexer)"]
    CE_AI --> CONTEXT7["Context7 (Modern Library & Docs Retrieval)"]
    CE_AI --> RTK["RTK (Rust Token Killer Output Filter)"]
    CE_AI --> SEQ["Sequential Thinking (Structured Reasoning Protocol)"]
```

### Key Ecosystem Tools:

1. **[`gentle-ai`](https://github.com/Gentleman-Programming)**:
   - The foundational suite and primary inspiration behind OpenSpec, Spec-Driven Development, and the Compound Engineering architecture.
2. **[`Engram`](https://github.com/Gentleman-Programming/engram)**:
   - Persistent memory sidecar powered by SQLite + FTS5 full-text search. Stores architecture decisions, bug fixes, and user preferences across compaction cycles and sessions.
3. **[`CodeGraph`](https://github.com/colbymchenry/codegraph)**:
   - Codebase intelligence sidecar. Indexes AST symbols, function callers, callees, and blast-radius impacts before broad filesystem searches.
4. **[`Context7`](https://github.com/upstash/context7)**:
   - Real-time documentation retrieval engine providing up-to-date framework APIs, libraries, and best-practice guidance for AI agents.
5. **[`RTK / Rust Token Killer`](https://github.com/rtk-ai/rtk)**:
   - CLI Token Reduction Engine. Intercepts raw terminal streams (`cargo test`, `git status`, `docker ps`), stripping noise and compressing text by **60% to 90%** before hitting LLM context.
6. **[`Sequential Thinking`](https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking)**:
   - Structured reasoning protocol that enables AI agents to decompose complex problems, form hypotheses, reflect on test outcomes, and refine solutions step-by-step.

---

## 👨‍🏫 Teacher's Guide: Understanding Proactive Workflow Observability (v0.6.0)

Think of an AI coding agent as a pilot flying a high-performance jet plane across complex software projects:

1. **The TUI Workflow Dashboard (`ce-ai tui` ➔ `Workflow` Tab)**:
   - *The Cockpit Instrument Panel*: Imagine flying blind without gauges showing speed, altitude, or fuel. The TUI Workflow Dashboard acts like your live cockpit display: it visualizes exactly which of the 7 Flywheel stages your AI agent is navigating, what subtask is active, and shows historical progress checkpoints saved to disk.

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
