# Technical Exploration: Work Readiness & Verification Advisory

## Investigation & Context Synthesis

### 1. Workflow Dual-Mode Topology in `ce-ai`

`ce-ai` supports two complementary execution modes (Issue #334, #340):
1. **Organic Driven Development (ODD)**:
   - Protocol for rapid, daily iteration.
   - Operates on a single recoverable task brief per feature located at `odd/tasks/<feature>.md`.
   - Contains Problem Statement, Definition of Done checklist, Guardrails, and Notes.
   - When scope or complexity grows, ODD tasks graduate to OpenSpec via `ce-ai graduate`.
2. **Compound Engineering (CE 7-Stage FSM)**:
   - Formal stage-gate development cycle:
     - Stage 1: Ideation (`ce-brainstorm`)
     - Stage 2: OpenSpec Definition (`proposal.md`, `exploration.md`, `design.md`, `spec.md`)
     - Stage 3: Execution Plan (`tasks.md`)
     - Stage 4: Implementation (`ce-work` / WorkTdd)
     - Stage 5: Verification (`ce-ai gate check`, CI matrix)
     - Stage 6: Knowledge Capture (`ce-compound`)
     - Stage 7: Git Shipping (`ce-commit-push-pr`)

### 2. Evaluated Readiness Query Options

| Approach | Latency | Transparency | Failure Impact | Verdict |
| :--- | :---: | :---: | :---: | :--- |
| **Option A: Opaque Binary Pass/Fail** (`is_ready`) | ~200ms | ❌ None (developer doesn't know what's missing) | Low | Rejected: Cannot distinguish missing tests from incomplete docs. |
| **Option B: Multi-Dimensional Semantic Questions** | ~250ms | ✅ High (individual dimension confidence scores) | Low | **Selected**: Surfaces granular actionable feedback (`DoD: 95%`, `Tests: 55%`, `Graduation: 30%`). |
| **Option C: Hard-Blocking Gate at Checkpoint** | ~300ms | Medium | ❌ High (false negatives halt rapid workflow) | Rejected: Probabilistic advice must never block deterministic operations. |

---

## Architectural Tradeoffs

### Context Construction Strategy
To avoid token bloat and keep decision evaluation latency under 400ms:
- Extract only relevant section excerpts:
  - Task brief or proposal summary (first 500 chars).
  - Open vs completed checklist items (`- [ ]` vs `- [x]`).
  - Active git diff summary (`git diff --stat`).
- Truncate prompt context strictly if working tree diff is massive.

### Graduation Heuristic for ODD
In ODD, the question `graduation_recommended` assesses whether:
- File modifications touch multiple distinct architectural subsystems.
- Scope expanded beyond the initial problem description.
- High-risk operations (auth, data persistence, network egress) were introduced.
This gives the developer an early, objective advisory trigger to run `ce-ai graduate`.

---

## Architectural Decision

Adopt **Option B** with:
1. Standardized dimension questions for ODD (`dod_satisfied`, `guardrails_respected`, `graduation_recommended`, `ready_to_close`).
2. Standardized dimension questions for CE Stage Transitions (`requirements_clear`, `implementation_complete`, `tests_sufficient`, `docs_complete`).
3. Status indicators: `Ready` ($\ge 80\%$), `Warning` ($\ge 60\%$), `NotReady` ($< 60\%$).
4. Strictly non-blocking advisory output.
