# 🎓 Quick Start Guide: Compound Engineering & `ce-ai` in Practice

Welcome to the beginner's Quick Start guide! Whether you are building a new feature, fixing a production bug, or simply researching a codebase, this guide explains step-by-step how to use **Compound Engineering** skills alongside **`ce-ai`**.

> 🌱 **Haven't installed `ce-ai` or run a slash command yet?** This guide assumes both are already done. Start with [Getting Started](getting-started.md) first, then come back here.

---

## 💡 The Core Philosophy: Workflow Orchestration + Methodology

- **`ce-ai`**: Open-source workflow orchestration and governance for Compound Engineering across AI coding agents. It gives your coding agent a durable way to turn ideas into verified work and make each completed change improve the next one, managing host configurations, scope isolation, and workflow state persistence (`ce-ai workflow`). See [CE-AI positioning](ce-ai-positioning.md).
- **Compound Engineering**: The engineering methodology (skills and guidelines) that ensures every task builds compounding knowledge so your codebase becomes cleaner, safer, and easier to maintain over time.

---

## 🚀 Workflow Selection Matrix

Choose your entry point based on your task type:

```mermaid
flowchart TD
    START[What are you working on?] --> CHOICE{Task Type}
    CHOICE -->|New Feature / Enhancement| SCENARIO_1[Scenario 1: Full 7-Stage Workflow]
    CHOICE -->|Bug Fix / Production Crash| SCENARIO_2[Scenario 2: Direct Entry via ce-debug]
    CHOICE -->|Research / Docs / Quick Chore| SCENARIO_3[Scenario 3: Fast-Track / Targeted Pass]
```

---

## 🛠️ Scenario 1: Building a New Feature or Enhancement

When introducing a new capability, UI screen, or major architectural change, follow the complete 7-stage lifecycle:

```mermaid
flowchart LR
    S1[1. ce-brainstorm] --> S2[2. Living Arch & Deltas]
    S2 --> S3[3. ce-plan]
    S3 --> S4[4. ce-work]
    S4 --> S5[5. ce-code-review]
    S5 --> S6[6. ce-compound]
    S6 --> S7[7. ce-commit-push-pr]
```

### Step-by-Step Flow:

1. **Stage 1: Ideation & Requirements**
   - Run `/ce-brainstorm` when requirements are fuzzy. Only run `/ce-ideate` first if even the technical approach is uncertain (see Section 7 FAQ 0).
   - *Goal*: Clarify scope, user constraints, and out-of-scope boundaries. Writes `docs/brainstorms/<date>-<name>-requirements.md` — disposable input for Stage 2 (retained on disk as raw history, never deleted by rule), never a second source of truth.

2. **Stage 2: Living Architecture & Contracts (`docs/arch/`) — Optional for Scope > 200 LOC**
   - Formalize architecture deltas under `docs/arch/deltas/<feature_name>/` (or legacy `openspec/changes/<feature_name>/`) when modifying schemas, multi-repo contracts, or public APIs.
   - Author the contract progressively:
     - `proposal.md`: Problem statement, scope boundaries, risk assessment, and success criteria.
     - `exploration.md`: Technical investigation, evaluated options, and trade-offs.
     - `design.md`: System architecture, struct definitions, data schemas, and API contracts.
     - `spec.md`: Formal behavioral rules in explicit `WHEN ... THEN ...` format.
   - *How to Instruct an AI Agent*: Provide the path `docs/arch/deltas/<feature_name>/` (or legacy `openspec/changes/`) to your AI harness. For bug fixes, minor tasks, or scope < 200 LOC, skip Stage 2 and proceed directly to `/ce-plan` or `/ce-work`.

3. **Stage 3: Technical Execution Plan (`docs/plans/` SSOT)**
   - Run `/ce-plan` (and optionally `/ce-doc-review` to audit plan rigor).
   - *Goal*: `/ce-plan` creates the executable implementation plan in `docs/plans/<date>-<feature_name>-plan.md` broken into vertical Scrum slices with explicit TDD triads (Red ➔ Green ➔ Refactor ≤ 200 LOC). The checkboxes (`- [x]`) in `docs/plans/` serve as the Single Source of Truth (SSOT) for progress tracking and FSM stage deduction. (Legacy `tasks.md` remains supported transparently via the dual-path reader). When you reach Stage 4, planning is complete.

4. **Stage 4: TDD & Work**
   - Run `/ce-work` (or `/ce-simplify-code` after implementing).
   - *Goal*: Write tests first (Red), implement code (Green), and refactor cleanly. Save progress checkpoints via `ce-ai workflow checkpoint`.

5. **Stage 5: Empirical Verification**
   - Run `/ce-code-review` plus the project's own verification suite — unit tests, linters and E2E gates as defined by your stack (e.g. `cargo test`, `npm test`, `pytest`).
   - *Goal*: Zero lint warnings under your project's configuration and a 100% green test suite.

6. **Stage 6: Knowledge Capture**
   - Run `/ce-compound`.
   - *Goal*: Capture non-obvious discoveries and architecture patterns in `docs/solutions/` and `CONCEPTS.md`.

7. **Stage 7: Git Delivery**
   - Run `/ce-commit-push-pr`.
   - *Goal*: Create a feature branch, commit with value-communicating message, and open a Pull Request.

---

## 🐞 Scenario 2: Fixing a Bug or Production Crash

When fixing a defect, you **do not** need to write feature briefs or architecture documents! You enter the workflow directly at Stage 4.

```mermaid
flowchart TD
    BUG[Bug / Failing Test] --> DEBUG[Direct Entry: ce-debug]
    DEBUG --> LOGS[Inspect Un-truncated Logs]
    LOGS --> TEST[Write Failing Reproducer Test]
    TEST --> FIX[Apply Upstream Root Cause Fix]
    FIX --> VERIFY[Stage 5: Verification - project quality gates]
    VERIFY --> COMPOUND[Stage 6: ce-compound Store Solution]
    COMPOUND --> SHIP[Stage 7: ce-commit-push-pr Ship Fix PR]
```

### Step-by-Step Flow:

1. **Direct Entry (Stage 4: Diagnosis)**
   - Run `/ce-debug`.
   - *Behavior*: Inspects error tracebacks, writes a minimal failing test case (Red), and applies a targeted fix to upstream logic (Green).

2. **Stage 5: Verification**
   - Re-run the failing reproducer plus the project's full verification suite to confirm zero regressions.

3. **Stage 6: Knowledge Capture**
   - Run `/ce-compound`.
   - *Goal*: Store the bug solution in `docs/solutions/` so Engram memory remembers the fix for future sessions.

4. **Stage 7: Git Delivery**
   - Run `/ce-commit-push-pr` to ship the fix on a `fix/<name>` branch.

---

## ⚡ Scenario 3: Bypassing / Fast-Tracking the Workflow

### ❓ Can I skip or fast-track the workflow?
**YES!** Not every task is a multi-file feature or bug fix. Compound Engineering provides targeted shortcuts for specialized tasks:

```mermaid
flowchart TD
    TASK[Task Type] -->|Research / Codebase Survey| RESEARCH[Use ce-ideate or Research Subagents]
    TASK -->|Adopt / Switch / Revisit a Tech Decision| POV[Use ce-pov]
    TASK -->|Updating Documentation / Solutions| DOCS[Use ce-compound / ce-compound-refresh]
    TASK -->|Trivial Typos / Single-Line Chores| CHORE[Use ce-commit Directly]

    RESEARCH -->|No Specs / PR Needed| DONE_1[Deliver Research Output]
    POV -->|Project-Grounded Verdict| DONE_4[Decision Recorded, No Mutation Required]
    DOCS -->|Direct Write| DONE_2[Update docs/solutions/]
    CHORE -->|Direct Commit| DONE_3[Commit & Push]
```

### 1. Research & Codebase Surveys
- **Use**: `/ce-ideate` or dispatch read-only research subagents.
- **Workflow**: Bypasses architecture specifications, execution plans, and PR delivery. Generates a summary report or dossier directly for the user.
- ⚠️ **Two distinct uses of `/ce-ideate` — do not confuse them**:
  - **Use A — Standalone research (this scenario)**: you want ideas or a survey as the *final deliverable*. Nothing feeds forward; full bypass.
  - **Use B — Upstream of a feature**: you run ideation because the approach is uncertain, then continue into `ce-brainstorm` → Stage 2. In that case it is NOT a bypass: the chosen idea and rejected alternatives distill into `exploration.md` (see Section 7, FAQ 1b).

### 2. Technology & Architecture Decisions
- **Use**: `/ce-pov`.
- **Workflow**: Bypasses formal specifications entirely for a single, decisive question — "should this project adopt/switch to/revisit X?" — judged against the current codebase, not in the abstract. Also fits a CVE, deprecation, or ecosystem shift landing mid-session: is it actually load-bearing for *this* project?
- ⚠️ **`/ce-ideate` vs `/ce-pov` — do not confuse them**: `/ce-ideate` is *divergent* — it generates and ranks several candidate directions when you don't yet know what to build. `/ce-pov` is *convergent* — it takes one specific external candidate you already have in mind (a library, a pattern, a platform) and returns a single decisive verdict grounded in this project. If you need options, use `/ce-ideate`; if you need a yes/no on one already-named option, use `/ce-pov`. If the verdict is "adopt," continue into `/ce-brainstorm` → Stage 2 like any other upstream idea.

### 3. Documentation Generation & Knowledge Audits
- **Use**: `/ce-compound` or `/ce-compound-refresh`.
- **Workflow**: Bypasses feature planning and TDD. Reads recent solutions or codebase state and writes directly to `docs/solutions/` or `CONCEPTS.md`.

### 4. Trivial Chores & Typo Fixes
- **Use**: Direct `/ce-commit` or `/ce-commit-push-pr`.
- **Workflow**: Bypasses Stages 1–3. Makes the minor edit, runs the project's relevant checks, and commits immediately.

---

## 🛠️ Advanced Workflow Patterns

### 1. Interrupting & Resuming Work Across Sessions

If you need to stop working and continue hours or days later:

1. **Before Stopping (Save Progress)**:
   ```bash
   ce-ai workflow checkpoint --stage 4 --task "4.2 Implementing Unit 2"
   ```
2. **When Resuming Later**:
   ```bash
   ce-ai workflow resume
   ```
3. **What Happens Behind the Scenes**:
   - `ce-ai` reads `state.json` and re-probes repository, manifest, and plan/architecture state to restore the recorded phase, active task string, and current checklist context. Engram retrieval, when configured in a harness, remains separate from this command.

---

### 2. Multi-Harness Collaboration (Harness Handoffs)

You are not locked into a single AI harness! Because `ce-ai` stores state in shared, standardized disk files (`state.json`, `docs/plans/`, `docs/arch/`, `docs/solutions/`, with backward compatibility for legacy `openspec/changes/`), different AI tools can handle different stages of the exact same task:

```mermaid
flowchart LR
    CLAUDE[Claude Code: Stage 1 & 2 Ideation & Architecture] --> CURSOR[Cursor: Stage 4 TDD & Code Editing]
    CURSOR --> AGY[Antigravity / OpenCode: Stage 5 & 6 Verification & ce-compound]
```

- **Example Workflow**:
  - Use **Claude Code** for Stage 1 (`ce-brainstorm`) and Stage 2 (`Living Architecture / Deltas`).
  - Use **Cursor** or **Copilot** for Stage 4 (`ce-work` / TDD code editing).
  - Use **Antigravity CLI (`agy`)** or **OpenCode** for Stage 5 (`Verify`), Stage 6 (`ce-compound`), and Stage 7 (`ce-commit-push-pr`).
- **Handoff Mechanism**: In any harness, simply execute `ce-ai workflow status` or `ce-ai workflow resume`. The active harness reads the shared state on disk and seamlessly continues from the previous tool's checkpoint.

---

### 3. Working with Multiple Git Worktrees (`ce-worktree`)

When working on multiple features or PRs concurrently, use isolated Git worktrees (`ce-worktree`):

- **Worktree Placement Rule**: Place worktrees as siblings under your project parent directory (e.g. `../my-repo-worktrees/feature-auth/`).
- **Workspace Scope Isolation**:
  Each worktree is an independent workspace directory. Initialize companions, adopt the project, and ensure harness versions are synced:
  ```bash
  codegraph init
  ce-ai init-prj
  ce-ai fleet sync
  ```
- **Workflow FSM Checkpoint Isolation**:
  `ce-ai workflow checkpoint`, `status`, and `resume` are automatically isolated per repository and worktree root in `~/.ce-ai/state.json`. Saving a checkpoint in one worktree will never overwrite or interfere with the active FSM stage or tasks in parallel worktrees on the same machine.
- **Gitignore Cleanliness**:
  Project adoption automatically maintains the sentinel block in `.gitignore` so local workstation paths, `.codegraph/`, and temporary harness artifacts never leak into Git.
- **CodeGraph & Engram Isolation**:
  - *CodeGraph*: Each worktree maintains its own independent index. Run `codegraph init` (or `ce-ai tools init codegraph`) inside the new worktree. Never copy or symlink `.codegraph/` between checkouts.
  - *Engram*: Memory observations are tagged with the workspace path so findings remain cleanly isolated per workspace.

### 4. Scenario 4: Starting a Project from Scratch (Greenfield Setup)

When creating a brand new project in an empty directory:

```mermaid
flowchart TD
    INIT["1. mkdir & git init"] --> ADOPT["2. codegraph init && ce-ai init-prj"]
    ADOPT --> FLEET["3. ce-ai fleet sync && ce-ai doctor"]
    FLEET --> STRAT["4. ce-strategy / ce-brainstorm"]
    STRAT --> SPEC["5. Living Architecture / Deltas in docs/arch/ (Optional)"]
    SPEC --> PLAN["6. ce-plan & ce-work (docs/plans/ SSOT & Test Pipeline)"]
    PLAN --> VERIFY["7. Verify linters & tests"]
    VERIFY --> COMP["8. ce-compound (Initialize CONCEPTS.md)"]
    COMP --> SHIP["9. ce-commit-push-pr (PR #1)"]
```

#### Step-by-Step Greenfield Process:

1. **Initialize Directory, Companions & Governance**:
   ```bash
   mkdir my-new-project && cd my-new-project
   git init
   codegraph init
   ce-ai init-prj
   ce-ai fleet pin latest && ce-ai fleet sync
   ce-ai doctor
   ```

2. **Define High-Level Product Strategy (Stage 1: Strategy)**:
   - Run `/ce-strategy`.
   - *Goal*: Establish tech stack (e.g. Rust, TypeScript, React), macro goals, and architectural principles in `STRATEGY.md`.

3. **Bridge from Strategy to Architecture: When to use `/ce-ideate` vs `/ce-brainstorm`**:
   - **Do I need `/ce-ideate` or `/ce-brainstorm` before drafting architecture specifications?**
     - **Option A (Feature Scope is Known)**: Run `/ce-brainstorm`.
       - *Why*: `/ce-brainstorm` refines your feature idea, resolves user constraints, and defines explicit out-of-scope boundaries. The output (`docs/brainstorms/`) feeds directly into drafting living specs in `docs/arch/` or the execution plan in `docs/plans/`.
      - **Option B (Uncertain Technical Approach or Architecture Alternatives)**: Run `/ce-ideate` first, then `/ce-brainstorm`.
        - *Why*: `/ce-ideate` generates and ranks mechanism-level directions with explicit rejection reasons in `docs/ideation/`. Once one direction is chosen, run `/ce-brainstorm` to lock requirements; the chosen idea and rejected alternatives distill into ADRs (`docs/arch/adr/`) or `exploration.md`.
     - **Option C (Standard Repository Scaffolding)**: Skip both and proceed directly to `/ce-plan` or initial scaffolding.
       - *Why*: Basic repository boilerplate (`Cargo.toml`, linter configs, directory tree) is standard and unambiguous.

4. **Draft Living Architecture & Contracts (Stage 2: Optional)**:
   - For architecture-defining greenfield projects, formalize domain specifications under `docs/arch/system/` (or temporary deltas in `docs/arch/deltas/`). For tactical scaffolding, skip directly to Step 5.

5. **Build Boilerplate & Test Pipeline (Stage 3 & 4: Plan & Work/TDD)**:
   - Run `/ce-plan` ➔ `/ce-work`. Build initial files, configure linters (`clippy`, `eslint`), and set up tests with progress tracked in `docs/plans/`.

6. **Establish Verification Gates (Stage 5: Verification)**:
   - Configure and run the project's test suite (`cargo test`, `npm test`, `pytest`, …) to keep it green.

7. **Capture Initial Architecture Concepts (Stage 6: Knowledge Capture)**:
   - Run `/ce-compound` to initialize `CONCEPTS.md` and document initial structural decisions in `docs/solutions/`.

8. **Ship First Commit (Stage 7: Git Delivery)**:
   - Run `/ce-commit-push-pr` to commit initial repository structure.

---

### 5. Alternative Development Methodologies: What if I don't use TDD?

While Test-Driven Development (TDD: Red-Green-Refactor) is the recommended default in Stage 4, **`ce-ai` and Compound Engineering fully support alternative development styles**:

```mermaid
flowchart TD
    CHOICE{"What is your development style?"}
    CHOICE -->|"TDD (Default)"| TDD["TDD: Write Failing Test Red ➔ Code Fix Green ➔ Refactor"]
    CHOICE -->|"Code-First"| CODE_FIRST["Code-First: Implement Code ➔ Write Tests ➔ Verify"]
    CHOICE -->|"BDD / Scenario-First"| BDD["BDD: Write WHEN-THEN Scenarios ➔ Implement ➔ Verify"]
    CHOICE -->|"Spike / R&D Spike"| SPIKE["R&D Spike: Quick PoC ➔ Evaluate ➔ Convert to Living Spec"]

    TDD --> GATE["Stage 5: Mandatory Verification - project quality gates"]
    CODE_FIRST --> GATE
    BDD --> GATE
    SPIKE -->|"If Retained"| GATE
```

#### How Non-TDD Variants Work in the FSM:

1. **Code-First + Post-Verification**:
   - *Flow*: Implement code directly in Stage 4 based on specification requirements (`spec.md` or living domain specs), then write tests afterwards.
   - *FSM Rule*: The FSM does not restrict whether code or test files were written first. However, **Stage 5 (Empirical Verification)** requires — by workflow contract and review policy, not by a CLI gate — that unit/integration tests exist and pass 100% green before shipping.

2. **Behavior-Driven Development (BDD)**:
   - *Flow*: Define acceptance scenarios using explicit `WHEN <condition> THEN <expected outcome>` blocks in Stage 2 (`Living Architecture / Deltas`). Tests are written to validate high-level behavior contracts.

3. **Spike Prototyping / R&D Spikes**:
   - *Flow*: Run `/ce-ideate` to generate and rank candidate directions (it produces an idea dossier in `docs/ideation/`, **not code**). Once one direction is chosen, build the throwaway PoC yourself or with `/ce-work` in a scratch branch — no tests or specs at this point.
   - *Transition*: If the spike proves viable, convert the learnings into Stage 2 (`Living Architecture / Deltas`) and proceed through standard verification. If not, discard the branch — the only durable output was the decision recorded in the dossier.

> ⚠️ **The Non-Negotiable Rule**: Regardless of whether you use TDD, Code-First, or BDD, **Stage 5 (Empirical Verification)** and **Stage 6 (`ce-compound`)** remain mandatory: code must be verified empirically via tests, and discoveries must be documented in `docs/solutions/`.

### 6. Practical Guide: How to Author Architecture & Delta Specifications

Living architecture and contracts reside under `docs/arch/` (with seamless backward compatibility for legacy `openspec/changes/` via `src/compat/arch.rs`):

```mermaid
flowchart TD
    DIR["1. Living Architecture: docs/arch/deltas/<feature>/ (or system/)"] --> FILES["2. Author Contract: proposal, exploration, design, spec"]
    FILES --> PROP["proposal.md: Problem & Scope Boundaries"]
    FILES --> EXPL["exploration.md / ADR: Tech Options & Trade-offs"]
    FILES --> DES["design.md: Schemas & Struct Definitions"]
    FILES --> SPEC["spec.md: WHEN...THEN Acceptance Scenarios"]

    FILES --> PLAN["3. ce-plan Generates docs/plans/ (Stage 3 SSOT)"]
    PLAN --> TASK["docs/plans/*.md: Executable Checklist (- [ ] Unit)"]
    TASK --> INSTRUCT["4. Instruct Agent: Execute via ce-work & track in docs/plans/"]
```

#### The Canonical Specification Components:

1. **`proposal.md`**:
   - Defines the problem statement, in-scope vs. out-of-scope boundaries, risk evaluation, and success criteria.
2. **`exploration.md` / ADR (`docs/arch/adr/`)**:
   - Documents technical research, evaluated libraries, architectural trade-offs, and prototype findings.
3. **`design.md`**:
   - Details technical architecture, data schemas, API contracts, struct fields, and error exit code mappings.
4. **`spec.md`**:
   - Written in formal `WHEN ... THEN ...` scenario blocks:
      ```markdown
      ### Scenario 1: Project Adoption
      WHEN the user runs `ce-ai init-prj` inside a Git repository
      THEN it MUST inject harness workflow files and configure companion hooks without clobbering user configs.
      ```
5. **Execution Plan (`docs/plans/`)** — *generated by `/ce-plan` in Stage 3*, serving as the SSOT:
   - Contains checkable tasks (`- [ ]`) broken into atomic implementation units:
      ```markdown
      - [ ] Unit 1: Add `--tier` CLI flag parsing in `src/commands/init_prj.rs`
      - [ ] Unit 2: Implement repository root resolution via `git rev-parse`
      - [ ] Unit 3: Add integration tests in `tests/cli.rs`
      ```

#### How to Instruct an AI Agent with Living Architecture:
- **Prompt Example**:
  > *"Execute the tasks defined in `docs/plans/2026-10-23-workspace-scope-plan.md` using the `/ce-work` skill. Update each checklist item to `- [x]` as tests pass."*

---

### 7. Transitioning from Spec-Driven Development (SDD / `gentle-ai`)

#### How does `ce-ai` handle projects migrating from SDD?

If your project previously used **Spec-Driven Development (SDD)** (such as `gentle-ai` or OpenSpec):

```mermaid
flowchart TD
    TRADITIONAL_SDD["Traditional SDD Workflow\n(Specs ➔ Plan ➔ Code ➔ Verify ➔ DONE)"] -->|"Missing Stage 6 Knowledge Capture"| SINK["Knowledge Loss & Repeated Bugs"]

    CE_WORKFLOW["Compound Engineering Workflow\n(Specs ➔ Plan ➔ Code ➔ Verify ➔ COMPOUND ➔ Flywheel)"] -->|"Stage 6: ce-compound"| FLYWHEEL["docs/solutions/ & CONCEPTS.md\n(Compounding Knowledge Flywheel)"]
```

- **100% Backward Compatibility**: `ce-ai` provides a dual-path architecture resolver. All legacy specifications in `openspec/changes/<feature_name>/` and `openspec/specs/` continue to work automatically without breakage.
- **Assisted Modernization**: Run `ce-ai init-prj --migrate-arch` to migrate legacy `openspec/` directories to canonical `docs/arch/` structures (`system/`, `adr/`, `archive/`).

#### The Mental Model: Architecture Defines Contracts — Plans in `docs/plans/` Track Progress

> 💡 **KISS · YAGNI · DRY**: Write each fact exactly once. Living architecture in `docs/arch/` defines *what* to build and system contracts; `docs/plans/` is the **single source of truth (SSOT)** for *progress tracking*. Every other skill either **feeds** them, **executes** them, or **learns from** them. If you find yourself copying the same content into two documents, stop — one of them should reference the other.

**Who touches what (and what each stage adds):**

| Stage | Skill | Reads (Input) | Writes (Output) | Role in Architecture & Progress |
| :--- | :--- | :--- | :--- | :--- |
| 0. Idea Discovery *(optional)* | `/ce-ideate` | Focus hint + repo scan | `docs/ideation/*.md` dossier | **Feeds it**: the chosen idea plus rejected alternatives distill into ADRs or `exploration.md`. Dossier is disposable input — retained as raw history, not a maintained document. Expensive (~9 sub-agents) — skip when the approach is already known |
| 1. Ideation | `/ce-brainstorm` | Your idea + constraints | `docs/brainstorms/*.md` | **Feeds it**: raw material to distill into `proposal.md` / `exploration.md` or directly into `docs/plans/` |
| 2. Living Arch & Deltas *(optional)* | You (+ agent) | Brainstorm doc | `docs/arch/deltas/` or `system/` | **Defines contracts**: frozen acceptance behavior for scope > 200 LOC |
| Gate | `/ce-doc-review` | Brainstorm or plan docs | Findings only | **Audits it**: flags gaps; never rewrites specs |
| 3. Execution Plan | `/ce-plan` | Specs / Brainstorms | `docs/plans/*.md` | **SSOT Progress Ledger**: generates vertical Scrum slices with TDD triads (`- [ ] Unit N`) |
| 4. Work | `/ce-work` | `docs/plans/*.md` | Code + green tests | **Executes it**: ticks `- [x]` items — `docs/plans/` is the progress ledger |
| 5. Verify | Tests / E2E | Code | Evidence | **Validates it**: proves every contract or test scenario green |
| 6. Capture | `/ce-compound` | Session learnings | `docs/solutions/`, `CONCEPTS.md` | **Extends beyond it**: knowledge the spec never contained |
| 7. Ship | `/ce-commit-push-pr` | Branch | PR + Archive | **Delivers it**: code + evidence + `ce-ai archive` promotes deltas |
| 5. Verify | Tests / E2E | Code | Evidence | **Validates it**: proves every `spec.md` WHEN/THEN scenario |
| 6. Capture | `/ce-compound` | Session learnings | `docs/solutions/`, `CONCEPTS.md` | **Extends beyond it**: knowledge the spec never contained |
| 7. Ship | `/ce-commit-push-pr` | Branch | PR | **Delivers it**: spec + code + evidence together |

**Newbie FAQ — "Where does the duplication go?"**

0. *"Do I always need `/ce-ideate` or `/ce-brainstorm` before formal architecture?"*
   No. KISS: if requirements AND approach are clear, write living architecture specs directly. Brainstorm only when requirements are fuzzy; ideate only when even the approach is uncertain (and remember it dispatches ~9 sub-agents — the most expensive skill in the pipeline).

1. *"If I already wrote `proposal.md`, does `/ce-brainstorm` duplicate it?"*
   No. Run brainstorm **before** Stage 2. Its conclusions get distilled into `proposal.md` under `docs/arch/deltas/<feature>/` (or legacy `openspec/changes/`); the brainstorm doc is disposable input — retained on disk as raw history, never a second source of truth.

1b. *"Doesn't `/ce-ideate` duplicate `exploration.md`?"*
   No — same distillation rule, different target. The ideation dossier (`docs/ideation/`) lists many ideas with critiques; `exploration.md` records only the ONE chosen direction plus one-line reasons for the rejected alternatives, linking back to the dossier:
   ```markdown
   # docs/arch/deltas/auth-refactor/exploration.md (or legacy openspec/changes/auth-refactor/exploration.md)
   Chosen: session-as-entity (survivor #1 of ideation run 2026-08-22)
   Rejected: JWT-stateless — revocation would require a token blacklist (docs/ideation/auth-ideas.md §critique)
   ```
   After this entry exists, the dossier is conversation history, not a maintained document.

2. *"Where does the progress checklist live?"*
   Under lean living architecture, the implementation plan in `docs/plans/<date>-<feature>-plan.md` is the **sole Single Source of Truth (SSOT)** for progress tracking and FSM stage deduction (`- [x]` checkboxes). This completely eliminates the need to author or maintain redundant `tasks.md` packages.

3. *"When I run `/ce-work`, which document does it follow?"*
   `/ce-work` directly follows the implementation plan in `docs/plans/`, executing vertical Scrum slices with TDD triads (Red ➔ Green ➔ Refactor) and updating checklist items to `- [x]` as automated tests pass.

4. *"What about the review/test loop and human QA?"*
   `/ce-code-review` findings send you back into Stage 4 iterations until tests are 100% green. Human QA happens after `/ce-compound`, before opening/merging the PR (Stage 7).

**Migration Steps** (from traditional SDD / legacy OpenSpec):
1. Run `ce-ai init-prj` in the repository (and `codegraph init` if not yet indexed).
2. Run `ce-ai init-prj --migrate-arch` to automatically migrate legacy `openspec/` directories to canonical `docs/arch/system/` and `docs/arch/archive/`.
3. Run `ce-ai fleet sync` to ensure active coding agent harnesses are synchronized.
4. Run `/ce-compound-refresh` once to audit historical learnings against the codebase.
5. Create new plans natively in `docs/plans/`, enjoying the automatic compounding flywheel!

---

## 📋 Quick Reference Cheat Sheet

| Task Goal | Entry Skill | Workflow Stages Used | Deliverable Output |
| :--- | :--- | :--- | :--- |
| **New Feature** | `/ce-brainstorm` | Full Stages 1 ➔ 7 | Plan (`docs/plans/`) + Implementation + Solution + PR |
| **Implementation Plan** | `/ce-plan` | Stage 3 | Numbered unit plan in `docs/plans/` (SSOT) |
| **Implement / TDD Execution** | `/ce-work` | Stage 4 | Code + green tests, `docs/plans/` items ticked |
| **Doc / Plan Quality Audit** | `/ce-doc-review` | Gate after Stages 1 & 3 | Review findings (no rewrites) |
| **Approach Uncertain (pre-feature)** | `/ce-ideate` | Stage 0/1 Sub-Loop ➔ Stage 2/3 | Idea dossier in `docs/ideation/` → distills into plan or ADR |
| **Standalone Research / Survey** | `/ce-ideate` / Subagents | Targeted Pass (bypass) | Research report for the user only |
| **Adopt / Switch / Revisit a Technology** | `/ce-pov` | Targeted Pass (bypass) | Decisive, project-grounded verdict — no options list |
| **Bug Fix / Crash Repair** | `/ce-debug` | Direct Entry: Stage 4 ➔ 7 | Reproducer Test + Fix + Solution + PR |
| **Refactoring Clean Code** | `/ce-simplify-code` | Stage 4 (Sub-Loop) | Non-behavioral code tidying |
| **Documentation Update** | `/ce-compound` | Targeted Pass (Stage 6) | Solution doc in `docs/solutions/` |
| **Trivial Chore / Typo** | Direct Edit | Fast-Track: Stage 4 ➔ 7 | Direct git commit |
