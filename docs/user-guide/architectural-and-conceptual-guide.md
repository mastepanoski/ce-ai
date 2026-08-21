# 🏛️ Architectural & Conceptual Guide: `ce-ai`

This guide presents a deep dive into the **software architecture, design patterns, and engineering principles** upon which `ce-ai` is built. Its goal is to explain the architectural *how* and *why* behind every system decision.

---

## 1. Multi-Harness Orchestration Architecture

### 📐 Architectural Concept
`ce-ai` is engineered around **complete adapter decoupling** via the `HarnessAdapter` trait in Rust.

```
                      ┌─────────────────────────┐
                      │    ce-ai CLI Engine     │
                      └────────────┬────────────┘
                                   │
                      ┌────────────┴────────────┐
                      │ HarnessAdapter (Trait)  │
                      └────┬───────────────┬────┘
                           │               │
        ┌──────────────────┴──┐         ┌──┴──────────────────┐
        │ ClaudeCodeAdapter   │         │  OpenCodeAdapter    │
        │ (.claude.json)      │         │  (opencode.json)    │
        └─────────────────────┘         └─────────────────────┘
        ┌─────────────────────┐         ┌─────────────────────┐
        │ CursorAdapter       │         │ GenericJsonAdapter  │
        │ (.cursorrules)      │         │ (Pi, Kimi, AGY...)  │
        └─────────────────────┘         └─────────────────────┘
```

### 💡 Architectural Rationale
- **Interface Abstraction**: Every AI agent harness (Claude Code, OpenCode, Cursor, Copilot, Kimi, Antigravity) uses heterogeneous file formats and configuration structures (JSON schemas, Markdown comment-delimited blocks, plugin array lists).
- **Single Responsibility Principle (SRP)**: The core engine of `ce-ai` has zero knowledge of editor-specific syntax or file structures. It delegates configuration merging exclusively to the matching `HarnessAdapter`.
- **Safe Extensibility (Open/Closed Principle)**: Adding support for a new AI harness requires only implementing the `HarnessAdapter` trait for the target harness, ensuring zero regressions in the core engine.

---

## 2. Scope Isolation & Hierarchy (Global vs. Workspace Scope)

### 📐 Architectural Concept
The system enforces a **Scope-Aware Configuration Hierarchy**.

- **Global Scope (`~/.config/` / `~/.claude.json`)**: User-level configuration layer housing preferences and tools available across all projects on a developer's machine.
- **Workspace Scope (`./.opencode/` / `./.cursorrules`)**: Repository-level configuration layer bounded to the current Git working tree, deterministically resolved via `git rev-parse --show-toplevel`.

### 💡 Architectural Rationale
- **Prevention of Rule Cross-Contamination**: Different repositories have distinct architectures, security policies, and coding conventions. Global-only rules would cause Rust backend directives to contaminate React frontend AI sessions.
- **Team Reproducibility**: Workspace scope allows skills and agent rules to be checked directly into Git. Any teammate or agent instance cloning the repository automatically inherits the exact same operational context.

---

## 3. Sidecars & Knowledge Protocol Architecture (MCP & Sidecar Pattern)

### 📐 Architectural Concept
`ce-ai` leverages the **Sidecar Pattern** coupled with the **Model Context Protocol (MCP)** infrastructure.

```
┌─────────────────────────────────────────────────────────────────┐
│                       AI Execution Agent                        │
└──────┬──────────────────────┬──────────────────────┬────────────┘
       │ (MCP)                │ (MCP)                │ (MCP)
┌──────┴──────────────┐ ┌─────┴──────────────┐ ┌─────┴──────────────┐
│  Engram Memory      │ │ CodeGraph Engine   │ │ Context7 / RTK     │
│  (Long-Term Store)  │ │ (Code Topology)    │ │ (Live Docs/Specs)  │
└─────────────────────┘ └────────────────────┘ └────────────────────┘
```

- **Engram**: Long-term persistent memory server storing findings, decisions, and past solutions across sessions.
- **CodeGraph**: Static indexing and blast-radius analysis engine exposing codebase call graphs.
- **Context7 & RTK**: Real-time documentation and knowledge providers.

### 💡 Architectural Rationale
- **Decoupling Reasoning from Persistence**: Large Language Models (LLMs) excel at real-time reasoning but lack durable, long-term memory.
- **Context Window Token Efficiency**: Rather than stuffing entire codebases or past histories into the prompt, Sidecars respond to targeted queries on demand via MCP, reducing token overhead and preventing context saturation.

---

## 4. Finite State Machine & Flow Resilience (Workflow FSM & Checkpointing)

### 📐 Architectural Concept
AI-assisted development in `ce-ai` is governed by a **Finite State Machine (FSM)** structuring the lifecycle into 7 deterministic stages:

$$\text{Ideation} \xrightarrow{1} \text{OpenSpec} \xrightarrow{2} \text{Plan} \xrightarrow{3} \text{Work/TDD} \xrightarrow{4} \text{Verify} \xrightarrow{5} \text{Compound} \xrightarrow{6} \text{Ship}$$

### 💡 Architectural Rationale
- **Determinism over Probability**: Code generation with LLMs is inherently probabilistic. Without an FSM enforcing formal specifications (`OpenSpec`), plans (`Plan`), and test-driven verification (`TDD`), execution degrades into superficial patches.
- **Checkpointing & Context Re-hydration**:
  - *Problem*: During long-running multi-file tasks, an LLM's context window undergoes compaction (loss of earlier context).
  - *Solution*: `ce-ai workflow checkpoint` atomically serializes the current FSM phase and active task to disk. Upon session restart or agent hand-off, `ce-ai workflow resume` reads the checkpoint and re-hydrates state without losing context or duplicating work.

---

## 5. System Integrity & Fault Tolerance (POSIX I/O Guarantees)

### 📐 Architectural Concept

1. **Atomic Disk Writes (`write_atomic`)**:
   - To prevent file corruption (`state.json`, `opencode.json`), disk mutations never write directly to the target file. Content is written to a temporary file (`.tmp`) on the same filesystem and swapped via an OS-level atomic `rename` call.

2. **Cryptographic SHA256 Drift Detection**:
   - `install-manifest.json` tracks SHA256 checksums for every managed asset. The `sync` engine performs a 3-way diff:
     - **Copy**: Missing files on disk.
     - **Restore**: Locally modified files exhibiting hash drift.
     - **Remove**: Stale or deprecated assets.

3. **User Configuration Preservation Principle**:
   - Non-destructive JSON mergers ensure `ce-ai` never deletes or clobbers custom user keys, third-party plugins, or MCP servers.

---

## 📊 Summary of Architectural Pillars

| Architectural Pillar | Design Pattern / Mechanism | System Problem Solved |
| :--- | :--- | :--- |
| **Adaptability** | `HarnessAdapter` (Traits) | Heterogeneous AI editors and configuration file formats. |
| **Scope Isolation** | Hierarchical Scope (Global vs. Workspace) | Cross-contamination of agent rules across different repositories. |
| **External Persistence** | Sidecars & MCP (Engram / CodeGraph) | Context window saturation and memory loss between sessions. |
| **Flow Determinism** | Workflow FSM & Checkpointing | Probabilistic instability and state loss post context compaction. |
| **Fault Tolerance** | Atomic Writes & SHA256 Manifest Indexing | File corruption during process crashes or unbuffered overwrites. |
