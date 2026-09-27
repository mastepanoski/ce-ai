<!-- Diátaxis Quadrant: Explanation | Audience: Senior -->
# 🎓 Why CE-AI Uses Spec-Driven Planning with Bounded Work Units Instead of Conversational Micro-TDD: Context Economics and Workflow Architecture for Coding Agents

> **Audience**: Senior / Contributor · **Intent**: Explanation
>
> This document explains the technical and architectural rationale behind `ce-ai`'s 7-stage workflow, OpenSpec execution baselines, and bounded work units. It analyzes how conversational agent loops interact with Test-Driven Development (TDD)—grounded in empirical findings from live agent MCP experiments—and why CE-AI positions TDD as an implementation technique within bounded units rather than as an end-to-end conversational orchestration protocol.

---

## 1. Cognitive Foundations vs. Agent Transcript Mechanics

Test-Driven Development (TDD), as formulated by Kent Beck, encourages developers to work in very small Red-Green-Refactor increments. 

One plausible cognitive benefit of these short feedback loops is that they reduce the amount of unresolved information a developer must keep in working memory at once. Human working memory is capacity-limited; while early literature proposed a capacity of "7 ± 2" items (Miller, 1956), subsequent research under controlled conditions estimates central working-memory capacity closer to 3–5 chunks, roughly 4 (Cowan, 2001). Although this provides a reasonable cognitive hypothesis for why small-step development helps human developers manage cognitive load, TDD was not established as a direct consequence of this neurological limit, nor is there strong empirical evidence that its cycle cadence was specifically designed around working-memory metrics.

Beyond cognitive load management, test-first development provides a distinct architectural advantage: **outside-in interface design**. By writing tests before implementation, the developer is forced to consume the component's API before writing its internals, ensuring the interface is shaped by the caller's needs rather than the implementer's convenience.

When AI coding agents (Claude Code, Cursor, OpenCode, Codex) emerged, workflows were frequently prompted to mimic this fine-grained micro-cadence:
> *"Work test-first, one test at a time: write one failing test, run it, see it fail, write the minimum code to pass, run tests, refactor, and repeat."*

While intuitive, transplanting human micro-TDD directly into an agent's conversational loop assumes that the agent's execution constraints mirror those of a human developer. An autoregressive transformer does not experience human working-memory limits, but it is strictly governed by **transcript context management, inference costs, and token processing economics**.

---

## 2. Empirical Evidence: Live Agent Loops in Smalltalk (Wilkinson Exp. 004)

The operational trade-offs of conversational micro-TDD were evaluated empirically in systematic research conducted by **Hernán Wilkinson** (co-founder of 10Pines, professor at the University of Buenos Aires, and Smalltalk/TDD researcher).

In the [ClaudeCode-CuisMCPServer study](https://github.com/hernanwilkinson/ClaudeCode-CuisMCPServer-Experiments) (comprising 356 autonomous sessions of Claude Code interacting with a live Cuis Smalltalk environment via Model Context Protocol tools), **Experiment 004 (*RefactoringToolsAndTDD*)** evaluated whether prompting Claude Code to follow strict micro-TDD produced better object-oriented design compared to *test-after* or *free* development techniques.

The experimental data reported the following results across conditions:

| Dimension | Cell C: Prescribed Micro-TDD | Cell D: Test-After | Cell E: Free Technique |
| :--- | :---: | :---: | :---: |
| **API Requests (Round-trips)** | **156** | **11** | **13** |
| **Input Tokens Processed** | **14,970 k** | **579 k** | **780 k** |
| **Financial Cost (USD)** | **$11.44** | **$2.26** | **$2.69** |
| **Execution Duration** | **1,415 s** (~24 min) | **571 s** (~9 min) | **681 s** (~11 min) |
| **AST Linter Findings (Mentor Defects)** | **43** (0.67 per method) | **102** (1.26 per method) | **69** (0.86 per method) |
| **Conditional Branches (`if` statements)** | **8** | **11** | **10** |
| **Test Smells** | **15** | **5** | **0** |

### Analytical Nuance: What the Data Shows (and What It Does Not)

1. **Substantial Interaction and Token Overhead**: In this experimental setup, prescribed conversational micro-TDD incurred roughly 5x higher financial cost, 26x more input token processing, and 14x more network round-trips compared to test-after and free techniques.
2. **Quality Trade-Off, Not Categorical Failure**: The data does not indicate that TDD "failed." On the contrary, the prescribed TDD condition **scored better on the study's design-quality proxies**: it reduced AST mentor design defects from 1.26 to 0.67 per method and reduced the number of conditional branches measured by the experiment (8 vs. 11/10). This indicates a genuine quality/cost trade-off rather than an inherent incompatibility with test-first thinking.
3. **Test Smell Emergence**: The prescribed TDD condition produced 15 test-smell findings versus 0 in the free condition. Importantly, the experiment does not isolate whether this was caused by transcript length, the specific TDD prompting strategy, the larger volume of generated test code, model-specific behaviors, or an interaction between these factors.

---

## 3. Context Economics and Transcript Dynamics in Agent Loops

Examining how conversational agent harnesses interact with fine-grained micro-cycles highlights three core architectural challenges:

```text
Conversational Micro-TDD (Turn-by-Turn Orchestration):
[Test 1 Fail] → [Code 1] → [Test 1 Pass] → ... → [Test 50 Fail] → [Code 50]
▲                                                                        │
└────────────── Multi-turn transcript accumulation ──────────────────────┘
                Total input token processing compounds with transcript depth

CE-AI Workflow Architecture (Decoupled Orchestration):
[Stage 2: OpenSpec Baseline] ──(Baseline)──► [Stage 3: Bounded Units (~200 LOC)]
                                                    │
                                                    ▼
                                        [Stage 4: Local Work & TDD]
                                        Local compiler & test runner (in-process)
                                        Context: curated durable task specification
```

### 1. Token Processing Dynamics in Append-Only Transcripts

In a naive append-only conversational harness where each turn adds approximately constant transcript content $\Delta$ and each subsequent inference consumes the accumulated transcript, total input-token processing compounds quadratically with the number of turns $n$:

$$T(n) = \sum_{i=1}^{n} (C + i\Delta) = O(n^2)$$

Modern harnesses employ mitigations such as prompt caching, transcript compaction, context pruning, subagents, and selective tool-output retention. However, even with prompt caching, each fine-grained conversational cycle still incurs cache-write latency and output generation costs. When an agent is prompted to execute dozens of micro-turns for minor code edits, cumulative processing and turnaround time scale significantly.

### 2. Context Utilization in Long Transcripts

Research on long-context language models (such as *Lost in the Middle*, Liu et al., 2023) demonstrates that models do not utilize all information within large context windows with uniform reliability. As transcripts grow:
- **Signal Competition**: High-priority task constraints and invariants must compete with accumulating historical artifacts (such as transient compiler errors from intermediate edits, verbose tool logs, and superseded outputs).
- **Holistic Context Degradation**: Focusing exclusively on making the immediate test pass within a crowded transcript can degrade the model's awareness of broader architectural patterns across the surrounding test suite.

### 3. Specification Drift and Correlated Epistemic Errors

When an agent generates requirements, test assertions, and application code simultaneously within an unstructured conversation, there is a risk of **specification drift**. If the model misinterprets an edge case or requirement, it may author both the test assertion and the implementation around the same flawed assumption. Both code and test agree, yet the behavior violates user intent.

Separating specification authoring (Stage 2) from code implementation (Stage 4) does not eliminate this risk entirely—a model could still make an error in the specification itself. However, it externalizes the requirement into a durable, inspectable document (`spec.md`), allowing human review, independent reviewer agents, or formal validation gates to inspect the contract before code is authored.

---

## 4. The CE-AI Approach: Spec-Driven Planning with Bounded Work Units

`ce-ai` addresses these challenges not by discarding test-first design discipline, but by **decoupling the orchestration loop from the implementation technique**:

```text
          Conversational Micro-TDD
                     │
              Red ↔ Green ↔ Refactor
                     │
         fine conversational turns


          CE-AI Workflow Architecture
                     │
            Durable Specification
            (Stage 2: OpenSpec Baseline)
                     ↓
             Execution Planning
            (Stage 3: tasks.md Decomposition)
                     ↓
             Bounded Work Unit (~200 LOC)
                     │
             ┌───────┴───────┐
             ↓               ↓
        Local TDD       Deterministic
      (Red-Green)       Verification
      in-process        (cargo/make)
             └───────┬───────┘
                     ↓
           Persisted Workflow State
                     ↓
             Next Work Unit
```

### 1. Specification as an Execution Baseline (Stage 2: OpenSpec)

Before implementation begins, Stage 2 captures the primary design benefit of TDD—**defining interfaces, behaviors, and invariants from the consumer's perspective**—in durable specification documents:
- `spec.md` specifies behavior using formal `WHEN ... THEN ...` clauses and explicit acceptance criteria.
- `design.md` defines types, contracts, and boundaries.

This specification serves as an **execution baseline**. It is not dogmatically immutable: if implementation in Stage 4 reveals an unworkable constraint or an incomplete requirement, the workflow protocol requires an explicit specification revision rather than silent drift during implementation.

### 2. Bounded Work Units as an Operational Heuristic

Rather than cycling through dozens of conversational round-trips for individual methods, Stage 3 ([`ce-plan`](file:///Users/mastepanoski/projects/web/ai/ce-ai/AGENTS.md)) decomposes implementation into bounded work units. 

`ce-ai` currently uses **~200 changed lines of code (LOC)** as a practical operational heuristic, not as a mathematically derived theoretical optimum. This heuristic balances several competing concerns:
- It keeps individual changes within a readable scope for human review.
- It provides a cohesive boundary for local unit and integration tests.
- It bounds the amount of code modified between state checkpoints.

Within each bounded unit, the agent or developer can use TDD freely. Crucially, the feedback loop runs **locally on developer tooling** (`cargo test`, `cargo clippy`, `make e2e`), keeping execution fast and independent of conversational round-trips.

### 3. Curated Durable Context vs. Accumulated Transcript History

Instead of relying on an append-only chat history that accumulates ephemeral error traces and tool outputs, `ce-ai` replaces conversational history with a curated set of durable artifacts:
- The active specification (`spec.md`).
- The system architectural invariants (`design.md`, `CONCEPTS.md`).
- The explicit task checklist (`tasks.md`).
- Persistent project memory via sidecars (such as Engram).

This structure ensures that each inference turn operates on curated, task-relevant context rather than an unmanaged transcript history. Under a fixed context budget per work unit, per-unit context overhead remains bounded ($O(1)$), allowing total processing across $n$ units to scale linearly ($O(n)$) rather than quadratically ($O(n^2)$).

### 4. Decoupled Workflow State and Context Reset Resilience

In conversational micro-TDD, task orientation is tied to the prompt transcript. If an API outage occurs or the context window compacts, the agent's progress can become ambiguous.

`ce-ai` separates three distinct types of state:
- **Workflow State**: Current development stage, active feature name, and completed task checkboxes (persisted deterministically in `state.json` and `tasks.md`).
- **Repository State**: Current git branch, unstaged modifications, and file drift (measured directly against the filesystem).
- **Reasoning State**: Transient model hypotheses, intermediate reasoning chains, and speculative ideas (ephemeral to the inference turn).

Because workflow state and repository drift are persisted outside the chat transcript, the conversational context can be cleared or compacted at any point without losing execution orientation. Upon resumption, `ce-ai workflow resume --json` reconstructs a compact, structured working context from durable artifacts, allowing the agent to resume execution from an explicit operational baseline.

---

## 5. Architectural Comparison

| Dimension | Human Micro-TDD (Beck) | Naive Conversational Micro-TDD | CE-AI Spec-Driven Architecture |
| :--- | :--- | :--- | :--- |
| **Primary Actor** | Human software engineer | LLM agent in chat/tool loop | LLM governed by deterministic CLI & FSM |
| **Role of TDD** | Core design & cognitive pacing discipline | Conversational orchestration protocol | Implementation technique within bounded units |
| **Cycle Cadence** | Minutes-level increments | 1–2 API turns per micro-change | Bounded work units (~200 LOC heuristic) |
| **Specification Timing** | Emerges incrementally through tests | Emerges incrementally (risk of drift) | Externalized baseline before execution (Stage 2) |
| **Feedback Mechanism** | Local test runner in IDE/terminal | Multi-turn conversational round-trips | Local compiler, test suite, and CI matrix |
| **Context Overhead** | Human working memory (~3–5 chunks) | Accumulates in conversational transcript ($O(n^2)$ naive) | Curated durable artifacts ($O(1)$ per unit, $O(n)$ total) |
| **Epistemic Validation** | Human review & runtime feedback | Same model generates test and code | External spec reviewable by humans or independent gates |
| **Context Reset Handling** | Developer memory / git status | Requires transcript recovery / compaction | Deterministic state recovery from `state.json` & `tasks.md` |

---

## 6. Synthesis

The relationship between Test-Driven Development and AI coding agents is not a binary choice between "TDD" and "Spec-Driven Development":

> **TDD's intellectual value lies in interface and behavior design prior to implementation. Its practical value in human workflows is managing cognitive load through small increments.**
>
> **For AI coding agents, using fine-grained micro-TDD as the conversational orchestration protocol incurs substantial interaction overhead, token inflation, and risk of attention degradation. CE-AI preserves the core benefits of test-first design through Stage 2 OpenSpec baselines, while shifting execution into bounded, locally verified work units that keep agent context focused, deterministic, and resilient.**
