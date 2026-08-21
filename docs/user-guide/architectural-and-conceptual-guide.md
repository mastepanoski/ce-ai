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

## 3. Sidecars & Knowledge Architecture (MCP Protocol vs. CLI Token Pre-Processors)

### 📐 Architectural Concept
`ce-ai` integrates two distinct architectural categories of companion tools:

```
┌──────────────────────────────────────────────────────────────────┐
│                      AI Execution Agent                          │
└──────────────┬───────────────────┬───────────────────┬───────────┘
               │ (MCP Protocol)    │ (MCP Protocol)    │ (CLI Wrapper)
┌──────────────┴───────┐ ┌─────────┴─────────┐ ┌───────┴──────────┐
│  Engram Memory       │ │ CodeGraph Engine │ │ RTK Token        │
│  (Long-Term Store)   │ │ (Call Graphs)     │ │ Reducer (Filter) │
└──────────────────────┘ └───────────────────┘ └──────────────────┘
```

1. **MCP Protocol Servers (Model Context Protocol)**:
   - **Engram**: Long-term persistent memory server storing findings, decisions, and past solutions across sessions via RPC.
   - **CodeGraph**: Static indexing and blast-radius analysis engine exposing codebase call graphs via MCP tools and CLI.
   - **Context7**: Real-time technical documentation and library spec provider.

2. **CLI Token Reduction Pre-Processor (RTK)**:
   - **RTK (Rust Token Killer / RTK-AI)** is **not an MCP server**, but a **CLI Output Filter / Command Pre-Processor**.
   - It intercepts verbose raw terminal outputs (e.g. `git status`, `cargo test`, `docker ps`, `ls -la`), strips boilerplate noise, and compresses the text stream by 60–90% *before* it reaches the LLM's context window.

### 💡 Architectural Rationale
- **Decoupling Reasoning from Storage**: LLMs excel at real-time reasoning but lack durable memory. MCP servers handle structured data retrieval without polluting main prompt state.
- **Context Window Cost & Speed Optimization**: Large raw CLI outputs exhaust context windows rapidly. Integrating CLI pre-processors like RTK reduces token consumption and cost while accelerating inference speed.

---

## 4. Finite State Machine & Compound Engineering Flywheel (Workflow FSM & Checkpointing)

### 📐 Architectural Concept
AI-assisted development in `ce-ai` is governed by a **Finite State Machine (FSM)** structuring the lifecycle into 7 deterministic stages directly enforcing the **Compound Engineering Flywheel**:

$$\text{Ideation} \xrightarrow{1} \text{OpenSpec} \xrightarrow{2} \text{Plan} \xrightarrow{3} \text{Work/TDD} \xrightarrow{4} \text{Verify} \xrightarrow{5} \text{Compound} \xrightarrow{6} \text{Ship}$$

### 💡 Architectural Rationale
- **Compound Engineering Alignment**: Compound Engineering dictates that software development must act as a self-reinforcing flywheel: every solved bug, design decision, and feature must compound knowledge over time. The FSM strictly mandates **Stage 6: Compound (`ce-compound`)**, ensuring agents document learnings in `docs/solutions/` and `CONCEPTS.md` before any task can close.
- **FSM Sub-Loops & Diagnostic Interrupts**:
  - *Exploration Sub-Loop (`ce-ideate` in Stage 1)*: Generates and evaluates unconstrained architectural ideas before transitioning to `ce-brainstorm` for structured requirements framing.
  - *Diagnostic Interrupt Sub-Loop (`ce-debug` in Stage 4/5)*: Triggered on test failure or bug detection. Freezes task state, enters an iterative diagnosis loop (hypothesis ➔ log extraction ➔ minimal reproducer ➔ root cause fix), and once verified, transitions control back to `ce-work` or `Verify` while tagging `ce-compound` if a non-obvious learning was uncovered.
  - *Refactoring Sub-Loop (`ce-simplify-code` in Stage 4)*: Executes non-behavioral code tidying and simplification passes post-Green TDD implementation.
- **Determinism over Probability**: Code generation with LLMs is inherently probabilistic. Without an FSM enforcing formal specifications (`OpenSpec`), plans (`Plan`), and test-driven verification (`TDD`), execution degrades into superficial patches.
- **Checkpointing, Cross-Session Resumption & Multi-Harness Handoffs**:
  - *Problem*: During long-running tasks or multi-harness workflows, an LLM's context window undergoes compaction or a developer switches editors (e.g. from Claude Code to Cursor or Antigravity).
  - *Solution*: `ce-ai workflow checkpoint` atomically serializes the FSM phase and active subtask to disk. Any harness running `ce-ai workflow resume` reads the shared disk state (`state.json` + Engram memory) and seamlessly continues work with 100% zero context loss.
- **Git Worktree Scope Isolation (`ce-worktree`)**:
  - *Problem*: Concurrent feature development across multiple Git worktrees can pollute shared configs or CodeGraph indices.
  - *Solution*: `ce-ai install --scope workspace` inside a worktree isolates managed skills (`./.opencode/`, `./.claude/`) to that worktree's path, while independent `.codegraph/` indices prevent call-graph corruption across worktrees.

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
