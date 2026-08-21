# 🎓 Masterclass: Understanding Finite State Machines (FSM) & Progress Checkpointing in AI Agents

Welcome to this dedicated step-by-step masterclass! If you are encountering terms like **Finite State Machine (FSM)**, **State Transitions**, **Context Compaction**, or **Progress Checkpointing** for the very first time, this guide is designed specifically for you.

We will explain these concepts from absolute scratch using everyday analogies, visual flowcharts, and concrete step-by-step walkthroughs.

---

## 1. The Core Problem: Why AI Agents Need Structure

### 🎲 Probabilistic vs. Deterministic Software

- **Human Software (Deterministic)**: A calculator always produces $2 + 2 = 4$. It follows exact, predictable rules.
- **Artificial Intelligence (Probabilistic)**: Large Language Models (LLMs) do not run rigid rules; they predict the most likely next tokens based on statistical probabilities.

### ❓ The Issue
When an AI agent is asked to build a large software feature, its probabilistic nature can lead to three common failures:
1. **Skipping Planning**: Jumping straight to modifying code without understanding requirements.
2. **Context Amnesia**: Forgetting what steps were completed after long conversations (due to **Context Window Compaction**).
3. **Inconsistent Quality**: Fixing one bug while introducing two regressions because there was no formal verification stage.

To solve this, `ce-ai` places the AI agent inside a **Finite State Machine (FSM)**.

---

## 2. What is a Finite State Machine (FSM)?

### 🚦 Everyday Analogy: The Traffic Light

Think of a standard traffic light:
- It has **3 States**: `RED`, `YELLOW`, `GREEN`.
- It follows **Strict Transition Rules**: `GREEN` can only transition to `YELLOW`, `YELLOW` can only transition to `RED`, and `RED` can only transition to `GREEN`.
- A traffic light **cannot** jump directly from `GREEN` to `RED` without passing through `YELLOW`.

```mermaid
stateDiagram-v2
    [*] --> Green
    Green --> Yellow: Timer Expires
    Yellow --> Red: Timer Expires
    Red --> Green: Timer Expires
```

### ⚙️ The FSM in `ce-ai`: The 7-Stage Development Cycle

In `ce-ai`, an AI agent is governed by an FSM that enforces a strict 7-stage lifecycle. The agent is **never** allowed to skip a stage.

```mermaid
flowchart LR
    S1[Stage 1: Ideation] --> S2[Stage 2: OpenSpec]
    S2 --> S3[Stage 3: Plan]
    S3 --> S4[Stage 4: Work / TDD]
    S4 --> S5[Stage 5: Verify]
    S5 --> S6[Stage 6: Compound]
    S6 --> S7[Stage 7: Ship / PR]
```

#### The 7 Stages Explained:

| Stage | Name | Real-World Analogy | What Happens Here? |
| :--- | :--- | :--- | :--- |
| **Stage 1** | **Ideation** (`ce-brainstorm`) | Architectural Blueprint Discussion | Exploring vague ideas, user constraints, and scope boundaries. |
| **Stage 2** | **OpenSpec Definition** | Formal Contract & Spec Sheet | Writing executable requirements (`proposal.md`, `spec.md`, `tasks.md`). |
| **Stage 3** | **Execution Plan** (`ce-plan`) | Construction Milestone Breakdown | Structuring implementation units, file lists, and test scenarios. |
| **Stage 4** | **TDD & Work** (`ce-work`) | Laying Bricks & Wiring | Writing tests first (Red), implementing code (Green), and refactoring. |
| **Stage 5** | **Verification** | Safety Inspector Audit | Running `cargo fmt`, `cargo clippy`, unit tests, and Docker E2E gates. |
| **Stage 6** | **Knowledge Capture** (`ce-compound`) | Updating the Operations Manual | Documenting discoveries in `docs/solutions/` and `CONCEPTS.md`. |
| **Stage 7** | **Git Shipping** (`ce-commit-push-pr`) | Delivering the Keys to the Client | Creating a feature branch, pushing to origin, and opening a Pull Request. |

---

## 3. What is a Checkpoint? (The "Savegame" Concept)

### 🎮 Everyday Analogy: Video Game Save Points

Imagine playing a massive 50-hour role-playing video game. Before facing a giant boss, you find a **Save Point**. 

If your console crashes or the power goes out, you don't lose 50 hours of effort—you reload your **Savegame** and resume from the exact minute you saved.

```mermaid
flowchart TD
    A[Agent Begins Work on Task 4.2] --> B[Task 4.2 Code & Tests Complete]
    B --> C[Trigger Checkpoint: ce-ai workflow checkpoint]
    C --> D[State Serialized to Disk in state.json]
    D --> E{Context Compaction or Crash?}
    E -->|Yes| F[New Agent Session / Re-hydration]
    F --> G[Run: ce-ai workflow resume]
    G --> H[Resumes at Task 4.3 with 100% Zero Data Loss]
```

### 🧠 Why AI Needs Checkpoints: Context Compaction

Large Language Models have a fixed memory limit called a **Context Window**. When a chat session becomes very long:
1. The AI interface automatically compresses earlier messages (**Context Compaction**).
2. The AI may lose track of which subtasks in `tasks.md` were completed and which are pending.

**The Solution**: By running `ce-ai workflow checkpoint`, the state is written directly to disk (`~/.ce-ai/state.json`) using atomic writes (`write_atomic`). Disk state survives session resets, compaction, and process terminations.

---

## 4. Step-by-Step Workflow Walkthrough

Let's walk through an entire development session step-by-step.

### 📍 Step 1: Querying Workflow Status (`ce-ai workflow status`)

Before starting or resuming work, the developer or agent queries the FSM engine:

```bash
ce-ai workflow status
```

**Output**:
```text
== [Workflow FSM & Progress Recovery Status] ==
7-Stage Cycle: [1:Ideation] ➔ [2:OpenSpec] ➔ [3:Plan] ➔ [4:Work/TDD] ➔ [5:Verify] ➔ [6:Compound] ➔ [7:Ship]
current phase: Stage 4: TDD & Work
active subtask: Unit 2: Companion Tools Manager (tasks.md 2/4 completed)
recovery status: Ready (100% state preserved)
```

---

### 📍 Step 2: Executing Work & Advancing States

The AI agent executes Stage 4 (TDD & Work):
1. Writes a unit test for the feature.
2. Implements the minimum code to make the test pass.
3. Updates `openspec/changes/<feature>/tasks.md` marking the checklist item as completed `[x]`.

---

### 📍 Step 3: Saving a Progress Checkpoint (`ce-ai workflow checkpoint`)

Before performing a heavy task, switching models, or ending a session, the agent calls:

```bash
ce-ai workflow checkpoint --phase "Stage 4: TDD & Work" --task "4.2 Companion Tools Manager Implemented"
```

**Output**:
```text
workflow: checkpoint saved successfully!
  phase: Stage 4: TDD & Work
  task: 4.2 Companion Tools Manager Implemented
```

**What happened behind the scenes?**
- `ce-ai` captured the phase name, active task string, and UTC timestamp.
- It invoked `crate::state::write_atomic`, writing to a `.tmp` file and atomically replacing `state.json`.
- Engram persistent memory recorded the session summary.

---

### 📍 Step 4: Re-Hydrating Context Post-Compaction (`ce-ai workflow resume`)

If the session is interrupted or a new agent instance (e.g. switching from Claude Code to Cursor, Gemini, or OpenCode) takes over:

```bash
ce-ai workflow resume
```

**Output**:
```text
workflow: resuming execution from latest checkpoint...
== [Workflow FSM & Progress Recovery Status] ==
7-Stage Cycle: [1:Ideation] ➔ [2:OpenSpec] ➔ [3:Plan] ➔ [4:Work/TDD] ➔ [5:Verify] ➔ [6:Compound] ➔ [7:Ship]
current phase: Stage 4: TDD & Work
active subtask: 4.2 Companion Tools Manager Implemented
workflow: re-hydrated context successfully. Proceeding with active task.
```

The new agent immediately knows **where it is**, **what was already built**, and **what needs to be done next**—without asking the user or re-doing completed work.

---

## 5. Summary Cheat Sheet

| Command | Everyday Analogy | Technical Purpose |
| :--- | :--- | :--- |
| `ce-ai workflow status` | Checking the Map | Queries current FSM stage and active subtask progress. |
| `ce-ai workflow checkpoint` | Saving Your Game | Persists stage name, task string, and timestamp atomically to disk. |
| `ce-ai workflow resume` | Reloading Your Save | Re-hydrates state and Engram memory after context compaction or agent hand-off. |
