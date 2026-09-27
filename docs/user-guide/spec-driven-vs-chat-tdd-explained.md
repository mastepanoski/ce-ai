<!-- Diátaxis Quadrant: Explanation | Audience: Senior -->
# 🎓 Why CE-AI Uses Spec-Driven Planning with Bounded Work Units Instead of Conversational Micro-TDD: Context Economics and Workflow Architecture for Coding Agents

> **Audience**: Senior / Contributor · **Intent**: Explanation
>
> This document explains the technical and architectural rationale behind `ce-ai`'s 7-stage workflow, OpenSpec execution baselines, and bounded work units. It analyzes how conversational agent loops interact with Test-Driven Development (TDD), draws on a small published experiment with Claude Code working through MCP tools, and explains why CE-AI positions TDD as an implementation technique within bounded work units rather than as the granularity of the agent's orchestration loop.

> **Core principle**: TDD remains an implementation technique within bounded work units rather than the granularity of the agent's orchestration loop.

---

## 1. Cognitive Foundations vs. Agent Transcript Mechanics

Test-Driven Development (TDD), as formulated by Kent Beck (*Test-Driven Development: By Example*, 2002), has developers work in very small Red-Green-Refactor increments: write a failing test, make it pass with the simplest code, then refactor. Design evolves incrementally through that feedback rather than being fixed upfront.

One plausible cognitive benefit of these short feedback loops is that they reduce the amount of unresolved information a developer must keep in working memory at once. Human working memory is capacity-limited; while early literature proposed a span of "7 ± 2" items (Miller, 1956), later work under controlled conditions estimates central working-memory capacity at roughly 3–5 chunks (Cowan, 2001). This is a reasonable hypothesis for why small-step development helps humans manage cognitive load, but TDD was not derived from this limit, and there is no strong evidence that its cadence was designed around it.

Beyond cognitive load, test-first development can provide a distinct design advantage: **consumer-oriented interface design**. By writing a test that exercises an API before implementing its internals, the developer uses the interface as its caller would, so it tends to be shaped by the caller's needs rather than the implementer's convenience.

When AI coding agents (Claude Code, Cursor, OpenCode, Codex) emerged, workflows were frequently prompted to reproduce this micro-cadence. The TDD instruction used in the experiment discussed below reads:
> *"Work test-first, one test at a time: write one failing test, run it and see it fail, write the minimum code that makes it pass, run all the tests, refactor while they stay green, and only then continue with the next test. Do not write production code without a failing test that asks for it."*

Transplanting human micro-TDD directly into an agent's conversational loop assumes the agent's constraints mirror a human developer's. An autoregressive transformer does not have human working-memory limits, but it is governed by **context management, inference cost, and token-processing economics**: in a conversational harness, each test run or edit that goes through a tool call is another model request over the accumulated context.

---

## 2. Empirical Evidence: Claude Code in a Live Cuis Smalltalk Image (Wilkinson, Experiment 004)

**Hernán Wilkinson** (co-founder of 10Pines, professor at FCEyN, University of Buenos Aires, and long-time Smalltalk and agile practitioner) published a repository of Claude Code experiments against a live Cuis Smalltalk image through an MCP server: [ClaudeCode-CuisMCPServer-Experiments](https://github.com/hernanwilkinson/ClaudeCode-CuisMCPServer-Experiments). At the time of writing it holds 356 Claude Code sessions across 21 experiments.

**[Experiment 004 (*RefactoringToolsAndTDD*)](https://github.com/hernanwilkinson/ClaudeCode-CuisMCPServer-Experiments/tree/main/experiments/004-RefactoringToolsAndTDD)** is 11 of those sessions (10 valid). Its hypothesis H2 asked whether, with refactoring tools and design heuristics available, TDD produces a better design than test-after or a free technique. The three relevant cells share scenario and configuration and differ only in the technique instruction:

- **Cell C**: TDD (the instruction quoted in §1)
- **Cell D**: test-after ("Implement the behavior first. Only when the implementation is complete, write the tests…")
- **Cell E**: free (no technique instruction)

Setup: model `claude-opus-5`, effort high, **one run per cell**, two greenfield exercises (*MineField* and *Aterrizar.com*). Each figure below is a single run, not an average.

| Measure | MineField C (TDD) | MineField D (test-after) | MineField E (free) | Aterrizar C (TDD) | Aterrizar D (test-after) | Aterrizar E (free) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **Cost (USD)** | 11.44 | 2.26 | 2.69 | 6.15 | 2.19 | 2.81 |
| **Input-side tokens** | 14,970 k | 579 k | 780 k | 6,826 k | 875 k | 958 k |
| **API requests** | 156 | 11 | 13 | 87 | 17 | 17 |
| **Test runs** | 81 | 1 | 1 | 32 | 2 | 4 |
| **Duration (s)** | 1,415 | 571 | 681 | 908 | 486 | 1,049 |
| **`if` statements in model** | 8 | 11 | 10 | 0 | 1 | 1 |
| **Mentor findings per method** | 0.67 | 1.26 | 0.86 | 0.65 | 0.59 | 0.64 |
| **Test smells** | 15 | 5 | 0 | 5 | 2 | 10 |

Source: the experiment's [README](https://github.com/hernanwilkinson/ClaudeCode-CuisMCPServer-Experiments/blob/main/experiments/004-RefactoringToolsAndTDD/README.md) and [table.md](https://github.com/hernanwilkinson/ClaudeCode-CuisMCPServer-Experiments/blob/main/experiments/004-RefactoringToolsAndTDD/table.md).

### What the Data Shows (and What It Does Not)

1. **Interaction and token overhead.** On MineField, TDD cost **5.1×** test-after and **4.3×** free, processed **25.9×** and **19.2×** the input-side tokens, and made **14.2×** and **12×** the API requests. On Aterrizar the ratios were smaller: **2.8×** and **2.2×** the cost, **7.8×** and **7.1×** the tokens, **5.1×** the requests against both. Wilkinson attributes the overhead to the technique text taken literally: one failing test at a time means one or two test-run calls per test, and each call is an API request that re-reads the context.
2. **Tokens and cost are not the same thing.** About 99% of TDD's input-side tokens on MineField were cache reads (14.82 M of 14.97 M). Prompt caching is why a ~26× token ratio became a ~5× cost ratio: caching reduces the price of re-reading the transcript, but not the number of requests or their latency.
3. **No consistent design advantage.** On MineField, TDD scored best on the design proxies (8 `if`s against 11 and 10; 0.67 mentor findings per method against 1.26 and 0.86). On Aterrizar the three cells were indistinguishable (0, 1 and 1 `if`s; 0.65, 0.59 and 0.64 findings per method). Wilkinson's own conclusion is that H2, better design with TDD, is **not supported by these measures**. He also notes that some mentor findings are layout habits that dominate some counts.
4. **Test smells go both ways.** TDD produced the most test smells on MineField (15 against 5 and 0), but on Aterrizar the free cell produced the most (10 against 5 and 2). The experiment cannot attribute test smells to the technique.
5. **Limits.** One run per cell, two exercises, no given tests on MineField (correctness there is the agent's own tests), one model, one language. These results illustrate a cost mechanism; they do not establish general effect sizes.

What CE-AI takes from this experiment is the cost mechanism, which is structural: when every Red-Green transition is a conversational round-trip, requests and context re-reads grow with the number of micro-cycles. It does not take a claim that TDD produces worse or better design.

---

## 3. Context Economics and Transcript Dynamics in Agent Loops

Examining how conversational agent harnesses interact with fine-grained micro-cycles highlights three architectural concerns:

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
                                        Local compiler & test runner
                                        Context: curated durable task specification
```

### 3.1 Token Processing in Append-Only Transcripts

In a naive append-only conversational harness where each turn adds approximately constant content $\Delta$ and each inference consumes the accumulated transcript plus a fixed base $C$ (system prompt, tool schemas), total input-token processing over $n$ turns grows quadratically:

$$T(n) = \sum_{i=1}^{n} (C + i\Delta) = nC + \Delta\frac{n(n+1)}{2} = O(n^2)$$

Modern harnesses mitigate this with prompt caching, transcript compaction, context pruning, subagents, and selective tool-output retention. Caching lowers the price of re-reading, as the experiment above shows, but each micro-cycle is still a separate request with its own latency, output generation, and a full pass over the context. The fixed base $C$ also matters: in Experiment 004, Wilkinson notes a 54 KB tool schema carried on every request.

### 3.2 Long-Context Retrieval Risk

As the transcript grows, reliable access to constraints distributed across the context becomes harder to assume. Long-context studies (e.g., *Lost in the Middle*, Liu et al., TACL 2024) show that information present in the context window is not used with uniform reliability, and that performance depends on where relevant information sits. In a long micro-TDD transcript, the task's constraints and invariants share the context with an accumulating residue of transient compiler errors, verbose tool logs, and superseded outputs.

This is a risk argument, not a measured effect: the cited research is about retrieval in long contexts in general, not about architectural awareness in coding agents specifically.

### 3.3 Specification Drift and Correlated Errors

When an agent generates requirements, test assertions, and application code within a single unstructured conversation, there is a risk of **specification drift**. If the model misreads an edge case or requirement, it may write both the test assertion and the implementation around the same flawed assumption: code and test agree, yet the behavior violates user intent.

Separating specification authoring (Stage 2) from implementation (Stage 4) does not eliminate this risk; the model can still err in the specification itself. It does externalize the requirement into a durable, inspectable document (`spec.md`) that human reviewers, independent reviewer agents, or validation gates can check before code is written.

---

## 4. The CE-AI Approach: Spec-Driven Planning with Bounded Work Units

`ce-ai` addresses these concerns not by discarding test-first discipline, but by **decoupling the orchestration loop from the implementation technique**:

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
   (Red-Green-Refactor)  Verification
                     (e.g., cargo/npm/pytest)
             └───────┬───────┘
                     ↓
           Persisted Workflow State
                     ↓
             Next Work Unit
```

Each layer preserves a different property:

```text
OpenSpec (Stage 2)        → external behavioral baseline
Bounded unit + local TDD  → Red → Green → Refactor at implementation level
Verification (Stage 5)    → conformance to the contract
```

### 4.1 Specification as an Execution Baseline (Stage 2: OpenSpec)

Before implementation begins, Stage 2 captures one important benefit associated with test-first development—making expected behavior and interfaces explicit from the consumer's perspective—while moving part of that design activity into a durable specification artifact:
- `spec.md` specifies behavior with `WHEN ... THEN ...` clauses and explicit acceptance criteria.
- `design.md` defines types, contracts, and boundaries.

OpenSpec is not TDD and does not replace it: it does not provide Red-Green-Refactor feedback. It provides the baseline that local TDD and verification are measured against.

The specification is an **execution baseline**, not an immutable document: if implementation in Stage 4 reveals an unworkable constraint or an incomplete requirement, the workflow requires an explicit specification revision rather than silent drift during implementation.

### 4.2 Bounded Work Units as an Operational Heuristic

Rather than cycling through dozens of conversational round-trips for individual methods, Stage 3 (`ce-plan`, see [AGENTS.md](../../AGENTS.md)) decomposes implementation into bounded work units.

`ce-ai` currently uses **~200 changed lines of code (LOC)** per unit as a practical operational heuristic, not as a derived optimum. It balances several concerns:
- It keeps individual changes within a readable scope for human review.
- It provides a cohesive boundary for local unit and integration tests.
- It bounds the amount of code modified between state checkpoints.

Within each unit, the agent or developer can use TDD freely. The aim is for fast feedback to come from **local developer tooling** (e.g., `cargo test`, `pytest`, `npm test`) and deterministic scripts, so that not every Red-Green-Refactor transition has to become a separate conversational inference turn.

### 4.3 Curated Durable Context vs. Accumulated Transcript History

Instead of relying on an append-only chat history that accumulates ephemeral error traces and tool outputs, `ce-ai` builds working context from a curated set of durable artifacts:
- The active specification (`spec.md`).
- Architectural invariants (`design.md`, `CONCEPTS.md`).
- The explicit task checklist (`tasks.md`).
- Persistent project memory via companion MCP servers (such as Engram).

Each unit therefore starts from curated, task-relevant context rather than an unmanaged transcript. Under a fixed context budget per work unit, per-unit context overhead remains bounded with respect to the number of previously completed units ($O(1)$), yielding $O(n)$ cumulative context processing across $n$ similarly bounded units. This bound is relative to the number of completed units, not to project size: if the artifacts loaded per unit (for example `design.md`) grow with the project, per-unit overhead grows with them, which is a reason to keep those artifacts concise.

### 4.4 Decoupled Workflow State and Context Reset Resilience

In conversational micro-TDD, task orientation lives in the transcript. If an API outage interrupts the session or the context is compacted, the agent's progress can become ambiguous.

`ce-ai` separates three kinds of state:
- **Workflow state**: current stage, active feature, and completed task checkboxes (persisted in `state.json` and `tasks.md`).
- **Repository state**: current git branch, uncommitted changes, and file drift (read directly from the filesystem).
- **Reasoning state**: transient hypotheses and intermediate reasoning (ephemeral to the inference turn).

Because workflow and repository state live outside the transcript, the conversation can be cleared or compacted without losing execution orientation. On resumption, `ce-ai workflow resume --json` reconstructs a compact, structured working context from those durable artifacts.

---

## 5. Architectural Comparison

| Dimension | Human Micro-TDD (Beck) | Naive Conversational Micro-TDD | CE-AI Spec-Driven Architecture |
| :--- | :--- | :--- | :--- |
| **Primary actor** | Human software engineer | LLM agent in chat/tool loop | LLM agent governed by a deterministic CLI and staged workflow |
| **Role of TDD** | Design and feedback discipline | Conversational orchestration protocol | Implementation technique within bounded units |
| **Cycle cadence** | Minutes-level increments | 1–2 API requests per micro-change | Bounded work units (~200 LOC heuristic) |
| **Behavioral baseline** | May exist externally (stories, acceptance criteria); executable design evolves incrementally through tests | May be explicit or emerge conversationally; vulnerable to drift if not externalized | Externalized baseline before execution (Stage 2) |
| **Feedback mechanism** | Local test runner in IDE/terminal | Multi-turn conversational round-trips | Local compiler, test suite, and CI |
| **Context overhead** | Human working memory (~3–5 chunks) | Accumulates in transcript ($O(n^2)$ naive) | Curated durable artifacts ($O(1)$ per unit w.r.t. completed units; $O(n)$ total) |
| **Epistemic validation** | Human review and runtime feedback | Same model writes test and code in one context | External spec reviewable by humans or independent gates |
| **Context reset handling** | Developer memory / git status | Transcript recovery or compaction | State recovery from `state.json` and `tasks.md` |

---

## 6. Synthesis

The relationship between TDD and AI coding agents is not a binary choice between "TDD" and "Spec-Driven Development", and CE-AI does not need to show that TDD lacks external specifications, or that it produces worse design, to justify its architecture.

> **For AI coding agents, using fine-grained micro-TDD as the conversational orchestration protocol can incur substantial interaction overhead and token-processing costs, while long transcripts introduce additional context-utilization risks. CE-AI therefore externalizes behavioral intent into durable OpenSpec baselines and uses bounded work units as the granularity of agent orchestration. TDD remains available inside those units as an implementation and design technique, while deterministic local tooling handles rapid verification without requiring every Red-Green-Refactor transition to become a conversational inference turn.**

---

## References

- Beck, K. (2002). *Test-Driven Development: By Example*. Addison-Wesley.
- Cowan, N. (2001). The magical number 4 in short-term memory: A reconsideration of mental storage capacity. *Behavioral and Brain Sciences*, 24(1), 87–114.
- Liu, N. F., et al. (2024). Lost in the Middle: How Language Models Use Long Contexts. *Transactions of the Association for Computational Linguistics*, 12, 157–173. (arXiv:2307.03172, 2023)
- Miller, G. A. (1956). The magical number seven, plus or minus two. *Psychological Review*, 63(2), 81–97.
- Wilkinson, H. (2026). *ClaudeCode-CuisMCPServer-Experiments*, Experiment 004 (*RefactoringToolsAndTDD*). https://github.com/hernanwilkinson/ClaudeCode-CuisMCPServer-Experiments/tree/main/experiments/004-RefactoringToolsAndTDD
